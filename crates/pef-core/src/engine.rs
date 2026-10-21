//! The audit, plan, apply and restore rules.

use crate::backup::BackupError;
use crate::catalog::{self, PrivacyPolicy};
use crate::error::Error;
use crate::model::{
    AuditFinding, AuditState, BACKUP_FORMAT_VERSION, PolicyChange, PolicyValue, PrivacyBackup,
};
use crate::store::PolicyStore;
use crate::time;

/// What an apply did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ApplyOutcome {
    /// Number of registry values written and verified.
    pub written: usize,
    /// Whether a recovery file was persisted. False only when there was nothing to change.
    pub backup_saved: bool,
}

/// What a restore did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RestoreOutcome {
    /// Number of values written back and verified.
    pub restored: usize,
    /// Number of entries that already held their original value.
    pub already_restored: usize,
    /// True for a dry run: nothing was written.
    pub dry_run: bool,
}

/// Runs the policy workflow against an injected [`PolicyStore`].
///
/// The engine never touches a platform directly, so every rule below is tested with the
/// in-memory store. Read-only methods take `&self`; only `apply` and `restore` can write.
#[derive(Debug)]
pub struct PrivacyEngine<S> {
    store: S,
}

impl<S: PolicyStore> PrivacyEngine<S> {
    /// Wraps a store.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// The underlying store.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// The underlying store, mutably. Used by tests and simulation to inject state.
    pub fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    /// Consumes the engine and returns its store.
    pub fn into_store(self) -> S {
        self.store
    }

    /// Compares each policy with its desired value. Never writes. A read failure or an
    /// unexpected registry type produces [`AuditState::Unknown`], never a pass.
    pub fn audit<'p>(
        &self,
        policies: impl IntoIterator<Item = &'p PrivacyPolicy>,
    ) -> Vec<AuditFinding>
    where
        'p: 'static,
    {
        policies
            .into_iter()
            .map(|policy| match self.store.read(policy) {
                Ok(current) => AuditFinding {
                    id: policy.id,
                    state: if current.exists && current.value == policy.desired {
                        AuditState::Configured
                    } else {
                        AuditState::DifferentOrNotConfigured
                    },
                    current: Some(current),
                    desired: policy.desired,
                    impact: policy.impact,
                    error: None,
                },
                Err(err) => AuditFinding {
                    id: policy.id,
                    state: AuditState::Unknown,
                    current: None,
                    desired: policy.desired,
                    impact: policy.impact,
                    error: Some(err.to_string()),
                },
            })
            .collect()
    }

    /// Reads every selected policy and builds the recovery plan for the ones that differ.
    ///
    /// Everything is read before anything could be written. An unreadable value or an
    /// unexpected registry type fails the whole plan rather than skipping the entry.
    pub fn plan<'p>(
        &self,
        policies: impl IntoIterator<Item = &'p PrivacyPolicy>,
        machine: &str,
    ) -> Result<PrivacyBackup, Error>
    where
        'p: 'static,
    {
        let mut changes = Vec::new();
        for policy in policies {
            let before = self.store.read(policy)?;
            if !before.exists || before.value != policy.desired {
                changes.push(PolicyChange {
                    id: policy.id.to_owned(),
                    before,
                    desired: policy.desired,
                });
            }
        }
        let backup = PrivacyBackup {
            version: BACKUP_FORMAT_VERSION,
            machine: machine.to_owned(),
            created: time::utc_now_rfc3339(),
            changes,
        };
        validate(&backup, machine)?;
        Ok(backup)
    }

    /// Applies a plan: validates it, checks nothing changed since planning, persists the
    /// recovery file through `persist`, then writes and verifies each value in order.
    ///
    /// If `persist` fails nothing is written. If a value changed between the check and
    /// its write, or a write does not read back, the operation stops with an error; the
    /// persisted file still restores everything that was written.
    pub fn apply(
        &mut self,
        plan: &PrivacyBackup,
        machine: &str,
        persist: impl FnOnce(&PrivacyBackup) -> Result<(), BackupError>,
    ) -> Result<ApplyOutcome, Error> {
        validate(plan, machine)?;
        if plan.changes.is_empty() {
            return Ok(ApplyOutcome::default());
        }
        self.ensure_unchanged(&plan.changes, false)?;
        persist(plan)?;
        let mut written = 0;
        for change in &plan.changes {
            let policy = catalog::get(&change.id)?;
            if self.store.read(policy)? != change.before {
                return Err(Error::ChangedDuringApply {
                    id: change.id.clone(),
                });
            }
            let desired = PolicyValue::present(change.desired);
            self.store.write(policy, desired)?;
            if self.store.read(policy)? != desired {
                return Err(Error::ReadBack {
                    id: change.id.clone(),
                });
            }
            written += 1;
        }
        Ok(ApplyOutcome {
            written,
            backup_saved: true,
        })
    }

    /// Restores a backup in reverse order, or only checks it when `dry_run` is true.
    ///
    /// Every entry must hold either its original value (already restored, skipped) or the
    /// enforced value (restored now). Anything else is a conflict and stops the operation
    /// before any write. Partial applies and repeated restores are therefore safe.
    pub fn restore(
        &mut self,
        backup: &PrivacyBackup,
        machine: &str,
        dry_run: bool,
    ) -> Result<RestoreOutcome, Error> {
        validate(backup, machine)?;
        self.ensure_unchanged(&backup.changes, true)?;
        let mut outcome = RestoreOutcome {
            dry_run,
            ..RestoreOutcome::default()
        };
        if dry_run {
            return Ok(outcome);
        }
        for change in backup.changes.iter().rev() {
            let policy = catalog::get(&change.id)?;
            let current = self.store.read(policy)?;
            if current == change.before {
                outcome.already_restored += 1;
                continue;
            }
            if current != PolicyValue::present(change.desired) {
                return Err(Error::ChangedDuringRestore {
                    id: change.id.clone(),
                });
            }
            self.store.write(policy, change.before)?;
            if self.store.read(policy)? != change.before {
                return Err(Error::RestoreReadBack {
                    id: change.id.clone(),
                });
            }
            outcome.restored += 1;
        }
        Ok(outcome)
    }

    fn ensure_unchanged(&self, changes: &[PolicyChange], restore: bool) -> Result<(), Error> {
        for change in changes {
            let current = self.store.read(catalog::get(&change.id)?)?;
            let acceptable = current == change.before
                || (restore && current == PolicyValue::present(change.desired));
            if !acceptable {
                return Err(Error::Conflict {
                    id: change.id.clone(),
                });
            }
        }
        Ok(())
    }
}

/// Checks a backup against the format version, this machine and the catalog.
///
/// Registry paths never come from the file: each entry is matched to the catalog by ID
/// and its desired value must equal the catalog's. Duplicate IDs, absent-but-non-zero
/// originals and more entries than the catalog holds are rejected.
pub fn validate(backup: &PrivacyBackup, machine: &str) -> Result<(), Error> {
    if backup.version != BACKUP_FORMAT_VERSION {
        return Err(Error::InvalidBackup(format!(
            "unsupported backup version {} (expected {BACKUP_FORMAT_VERSION}).",
            backup.version
        )));
    }
    if backup.machine != machine {
        return Err(Error::InvalidBackup(format!(
            "the backup was created on '{}' but this computer is '{machine}'.",
            backup.machine
        )));
    }
    if backup.changes.len() > catalog::all().len() {
        return Err(Error::InvalidBackup(format!(
            "{} entries exceed the {} policies in the catalog.",
            backup.changes.len(),
            catalog::all().len()
        )));
    }
    let mut seen: Vec<&str> = Vec::with_capacity(backup.changes.len());
    for change in &backup.changes {
        if seen.contains(&change.id.as_str()) {
            return Err(Error::InvalidBackup(format!(
                "duplicate entry for '{}'.",
                change.id
            )));
        }
        seen.push(&change.id);
        let policy =
            catalog::get(&change.id).map_err(|err| Error::InvalidBackup(format!("{err}")))?;
        if change.desired != policy.desired {
            return Err(Error::InvalidBackup(format!(
                "entry '{}' enforces {} but the catalog enforces {}.",
                change.id, change.desired, policy.desired
            )));
        }
        if !change.before.exists && change.before.value != 0 {
            return Err(Error::InvalidBackup(format!(
                "entry '{}' is marked absent but carries a value.",
                change.id
            )));
        }
    }
    Ok(())
}
