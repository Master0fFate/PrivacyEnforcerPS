//! The desktop session: everything the views show and every action they can take,
//! with no user-interface types so it can be tested directly.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use pef_core::{
    ApplyOutcome, AuditFinding, AuditSummary, Category, Command, MemoryStore, PolicyStore,
    PolicyValue, PrivacyBackup, PrivacyEngine, PrivacyPolicy, RestoreOutcome, backup, catalog,
};

/// Facts about the machine the session runs on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemStatus {
    /// One line describing the operating system, or why policy access is unavailable.
    pub description: String,
    /// Whether the process may write machine-wide policies.
    pub elevated: bool,
    /// True when values live in memory because there is no supported Windows registry.
    pub simulated: bool,
    /// The computer name recorded in backups.
    pub machine: String,
}

/// Tone of a status message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    /// Neutral information.
    Info,
    /// Something completed and was verified.
    Success,
    /// Something needs attention before continuing.
    Warning,
    /// Something stopped.
    Error,
}

/// A status message shown in the interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// Tone.
    pub kind: NoticeKind,
    /// Short headline.
    pub title: String,
    /// Longer explanation.
    pub detail: String,
}

impl Notice {
    fn new(kind: NoticeKind, title: impl Into<String>, detail: impl Into<String>) -> Notice {
        Notice {
            kind,
            title: title.into(),
            detail: detail.into(),
        }
    }
}

/// A backup file that was loaded, validated and dry-run checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedBackup {
    /// Where it came from.
    pub path: PathBuf,
    /// Its content.
    pub backup: PrivacyBackup,
}

/// The session state.
pub struct Session {
    engine: PrivacyEngine<Box<dyn PolicyStore>>,
    status: SystemStatus,
    selection: BTreeSet<&'static str>,
    findings: Vec<AuditFinding>,
    audited: bool,
    plan: Option<PrivacyBackup>,
    loaded: Option<LoadedBackup>,
    notice: Option<Notice>,
}

impl Session {
    /// A session over an arbitrary store.
    pub fn new(store: Box<dyn PolicyStore>, status: SystemStatus) -> Session {
        Session {
            engine: PrivacyEngine::new(store),
            status,
            selection: BTreeSet::new(),
            findings: Vec::new(),
            audited: false,
            plan: None,
            loaded: None,
            notice: None,
        }
    }

    /// A session over the real registry when the platform gate passes, otherwise a
    /// clearly labelled in-memory simulation.
    pub fn from_platform() -> Session {
        let machine = pef_platform::machine_name();
        match pef_platform::require_supported_windows()
            .and_then(|version| pef_platform::open_policy_store().map(|store| (version, store)))
        {
            Ok((version, store)) => Session::new(
                store,
                SystemStatus {
                    description: version.describe(),
                    elevated: pef_platform::is_elevated(),
                    simulated: false,
                    machine,
                },
            ),
            Err(err) => Session::simulated(err.to_string(), machine),
        }
    }

    /// An in-memory session for development, demos and tests.
    pub fn simulated(reason: impl Into<String>, machine: impl Into<String>) -> Session {
        let store = MemoryStore::with_values([
            ("advertising-id", PolicyValue::present(1)),
            ("clipboard-sync", PolicyValue::present(1)),
            ("app-camera", PolicyValue::present(0)),
        ]);
        Session::new(
            Box::new(store),
            SystemStatus {
                description: reason.into(),
                elevated: false,
                simulated: true,
                machine: machine.into(),
            },
        )
    }

    /// Facts about the machine.
    pub fn status(&self) -> &SystemStatus {
        &self.status
    }

    /// Whether writes are possible: elevated on Windows, or simulated.
    pub fn can_write(&self) -> bool {
        self.status.simulated || self.status.elevated
    }

    /// The latest audit, empty until [`Session::run_audit`] runs.
    pub fn findings(&self) -> &[AuditFinding] {
        &self.findings
    }

    /// Whether an audit has run in this session.
    pub fn audited(&self) -> bool {
        self.audited
    }

    /// Tallies of the latest audit.
    pub fn summary(&self) -> AuditSummary {
        AuditSummary::of(&self.findings)
    }

    /// The finding for one policy, if audited.
    pub fn finding(&self, id: &str) -> Option<&AuditFinding> {
        self.findings.iter().find(|finding| finding.id == id)
    }

    /// Reads every policy. Never writes.
    pub fn run_audit(&mut self) {
        self.findings = self.engine.audit(catalog::all());
        self.audited = true;
        let summary = self.summary();
        self.notice = Some(if summary.unknown > 0 {
            Notice::new(
                NoticeKind::Warning,
                format!("{} value(s) could not be read", summary.unknown),
                "Unknown never counts as protected. Check the error on each entry; an unexpected registry type is left untouched.",
            )
        } else {
            Notice::new(
                NoticeKind::Info,
                format!(
                    "{} of {} policies configured",
                    summary.configured,
                    summary.total()
                ),
                "Configured confirms the stored registry value only, not effective enforcement by every app.",
            )
        });
    }

    /// Whether a policy is selected for enforcement.
    pub fn is_selected(&self, id: &str) -> bool {
        self.selection.contains(id)
    }

    /// Number of selected policies.
    pub fn selected_count(&self) -> usize {
        self.selection.len()
    }

    /// The selected policies in catalog order.
    pub fn selection(&self) -> Vec<&'static PrivacyPolicy> {
        catalog::all()
            .iter()
            .filter(|policy| self.selection.contains(policy.id))
            .collect()
    }

    /// Selects or deselects one policy. Any preview is discarded.
    pub fn set_selected(&mut self, id: &str, selected: bool) {
        if let Ok(policy) = catalog::get(id) {
            if selected {
                self.selection.insert(policy.id);
            } else {
                self.selection.remove(policy.id);
            }
            self.plan = None;
        }
    }

    /// Flips one policy.
    pub fn toggle(&mut self, id: &str) {
        let selected = !self.is_selected(id);
        self.set_selected(id, selected);
    }

    /// Adds every policy of a preset to the selection.
    pub fn select_preset(&mut self, preset_id: &str) -> Result<usize, String> {
        let preset = catalog::preset(preset_id).map_err(|err| err.to_string())?;
        let policies = catalog::expand_preset(preset);
        for policy in &policies {
            self.selection.insert(policy.id);
        }
        self.plan = None;
        Ok(policies.len())
    }

    /// Selects or deselects every policy of a category.
    pub fn set_category_selected(&mut self, category: Category, selected: bool) {
        for policy in catalog::in_category(category) {
            if selected {
                self.selection.insert(policy.id);
            } else {
                self.selection.remove(policy.id);
            }
        }
        self.plan = None;
    }

    /// Whether every policy of a category is selected.
    pub fn category_fully_selected(&self, category: Category) -> bool {
        catalog::in_category(category).all(|policy| self.selection.contains(policy.id))
    }

    /// Clears the selection and any preview.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
        self.plan = None;
    }

    /// The current enforcement preview.
    pub fn plan(&self) -> Option<&PrivacyBackup> {
        self.plan.as_ref()
    }

    /// Builds the enforcement preview for the selection. Never writes.
    pub fn preview_enforce(&mut self) -> Result<&PrivacyBackup, String> {
        let selection = self.selection();
        if selection.is_empty() {
            self.plan = None;
            return Err("Select at least one policy first.".into());
        }
        let plan = self
            .engine
            .plan(selection, &self.status.machine)
            .map_err(|err| err.to_string())?;
        self.plan = Some(plan);
        Ok(self.plan.as_ref().expect("just set"))
    }

    /// Why an apply cannot start right now, if anything.
    pub fn apply_blocker(&self) -> Option<String> {
        if !self.can_write() {
            return Some(
                "Not elevated: this window can audit and preview only. Close it and run Privacy Enforcer as administrator to apply or restore."
                    .into(),
            );
        }
        match &self.plan {
            None => Some("Preview the selection first.".into()),
            Some(plan) if plan.changes.is_empty() => Some(
                "Nothing to change: every selected policy already holds its desired value.".into(),
            ),
            Some(_) => None,
        }
    }

    /// Applies the previewed plan after the typed confirmation, writing the recovery
    /// file first. Re-audits on success.
    pub fn apply_enforce(
        &mut self,
        backup_path: &Path,
        typed: &str,
    ) -> Result<ApplyOutcome, String> {
        if let Some(blocker) = self.apply_blocker() {
            return Err(blocker);
        }
        check_confirmation(Command::Enforce, typed)?;
        if backup_path.as_os_str().is_empty() {
            return Err("Choose a new backup file in a private folder first.".into());
        }
        let plan = self.plan.clone().expect("apply_blocker checked the plan");
        let path = std::path::absolute(backup_path).unwrap_or_else(|_| backup_path.to_path_buf());
        let result = self.engine.apply(&plan, &self.status.machine, |plan| {
            backup::save(&path, plan)
        });
        self.run_audit();
        match result {
            Ok(outcome) => {
                self.plan = None;
                self.notice = Some(Notice::new(
                    NoticeKind::Success,
                    format!("{} value(s) written and verified", outcome.written),
                    format!(
                        "Recovery file: {}. Keep it private. Restart affected apps; some policies need sign-out or restart. No services, security controls or network settings were changed.",
                        path.display()
                    ),
                ));
                Ok(outcome)
            }
            Err(err) => {
                let detail = if err.is_pre_write() {
                    "No policy values were written.".to_owned()
                } else {
                    format!(
                        "The operation began before it stopped. Keep {} and restore from it after auditing.",
                        path.display()
                    )
                };
                self.notice = Some(Notice::new(NoticeKind::Error, err.to_string(), detail));
                Err(err.to_string())
            }
        }
    }

    /// The loaded backup, if any.
    pub fn loaded_backup(&self) -> Option<&LoadedBackup> {
        self.loaded.as_ref()
    }

    /// Loads, validates and dry-run checks a backup file. Never writes.
    pub fn load_backup(&mut self, path: &Path) -> Result<&LoadedBackup, String> {
        self.loaded = None;
        let loaded = backup::load(path).map_err(|err| err.to_string())?;
        pef_core::validate(&loaded, &self.status.machine).map_err(|err| err.to_string())?;
        self.engine
            .restore(&loaded, &self.status.machine, true)
            .map_err(|err| err.to_string())?;
        self.loaded = Some(LoadedBackup {
            path: path.to_path_buf(),
            backup: loaded,
        });
        Ok(self.loaded.as_ref().expect("just set"))
    }

    /// Why a restore cannot start right now, if anything.
    pub fn restore_blocker(&self) -> Option<String> {
        if !self.can_write() {
            return Some(
                "Not elevated: this window can audit and preview only. Close it and run Privacy Enforcer as administrator to apply or restore."
                    .into(),
            );
        }
        match &self.loaded {
            None => Some("Open and validate a backup file first.".into()),
            Some(loaded) if loaded.backup.changes.is_empty() => {
                Some("The backup records no changes; there is nothing to restore.".into())
            }
            Some(_) => None,
        }
    }

    /// Restores the loaded backup after the typed confirmation. Re-audits afterwards.
    pub fn apply_restore(&mut self, typed: &str) -> Result<RestoreOutcome, String> {
        if let Some(blocker) = self.restore_blocker() {
            return Err(blocker);
        }
        check_confirmation(Command::Restore, typed)?;
        let loaded = self.loaded.clone().expect("restore_blocker checked");
        let result = self
            .engine
            .restore(&loaded.backup, &self.status.machine, false);
        self.run_audit();
        match result {
            Ok(outcome) => {
                self.notice = Some(Notice::new(
                    NoticeKind::Success,
                    format!(
                        "{} value(s) restored, {} already held their original value",
                        outcome.restored, outcome.already_restored
                    ),
                    "Every write was read back and verified. Keep the backup file until you have confirmed the effective behavior.",
                ));
                Ok(outcome)
            }
            Err(err) => {
                let detail = if err.is_pre_write() {
                    "No policy values were written.".to_owned()
                } else {
                    "The restore began before it stopped. Audit, then run the restore again; entries already restored are skipped.".to_owned()
                };
                self.notice = Some(Notice::new(NoticeKind::Error, err.to_string(), detail));
                Err(err.to_string())
            }
        }
    }

    /// The current status message.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
    }

    /// Replaces the status message.
    pub fn set_notice(
        &mut self,
        kind: NoticeKind,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.notice = Some(Notice::new(kind, title, detail));
    }

    /// Removes the status message.
    pub fn clear_notice(&mut self) {
        self.notice = None;
    }

    /// Direct access to the store, for tests.
    pub fn store(&self) -> &dyn PolicyStore {
        self.engine.store().as_ref()
    }
}

fn check_confirmation(command: Command, typed: &str) -> Result<(), String> {
    let word = command
        .confirmation_word()
        .expect("write commands have a word");
    if typed == word {
        Ok(())
    } else {
        Err(format!(
            "Type {word} exactly to confirm. Nothing was written."
        ))
    }
}
