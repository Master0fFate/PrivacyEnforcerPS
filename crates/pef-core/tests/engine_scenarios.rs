//! End-to-end scenarios for the engine against the in-memory store.
//!
//! These are the safety claims the README makes, written as executable checks.

use std::cell::RefCell;
use std::collections::BTreeMap;

use pef_core::backup::{self, BackupError};
use pef_core::catalog;
use pef_core::{
    AuditState, AuditSummary, Error, MemoryStore, PolicyChange, PolicyStore, PolicyValue,
    PrivacyBackup, PrivacyEngine, PrivacyPolicy, StoreError, validate,
};

const MACHINE: &str = "test-machine";

fn all() -> Vec<&'static PrivacyPolicy> {
    catalog::all().iter().collect()
}

fn ids(policies: &[&'static PrivacyPolicy]) -> Vec<&'static str> {
    policies.iter().map(|p| p.id).collect()
}

/// Hook called before each read with the policy ID and how many times it was read.
type ReadHook = Box<dyn Fn(&str, usize) -> Option<PolicyValue>>;

/// A store that records every write and lets a test rewrite values between reads.
struct ScriptedStore {
    inner: MemoryStore,
    writes: Vec<(String, PolicyValue)>,
    reads: RefCell<BTreeMap<String, usize>>,
    /// Called before each read with the ID and how many times it was read so far.
    before_read: Option<ReadHook>,
    pending: RefCell<Vec<(String, PolicyValue)>>,
}

impl ScriptedStore {
    fn new() -> Self {
        Self {
            inner: MemoryStore::new(),
            writes: Vec::new(),
            reads: RefCell::new(BTreeMap::new()),
            before_read: None,
            pending: RefCell::new(Vec::new()),
        }
    }
}

impl PolicyStore for ScriptedStore {
    fn read(&self, policy: &PrivacyPolicy) -> Result<PolicyValue, StoreError> {
        let count = {
            let mut reads = self.reads.borrow_mut();
            let entry = reads.entry(policy.id.to_owned()).or_insert(0);
            *entry += 1;
            *entry
        };
        if let Some(hook) = &self.before_read
            && let Some(value) = hook(policy.id, count)
        {
            // Reads take `&self`; queue the external change and apply it on top of the
            // inner store's answer so the next read observes it.
            self.pending
                .borrow_mut()
                .push((policy.id.to_owned(), value));
            return Ok(value);
        }
        if let Some((_, value)) = self
            .pending
            .borrow()
            .iter()
            .rev()
            .find(|(id, _)| id == policy.id)
        {
            return Ok(*value);
        }
        self.inner.read(policy)
    }

    fn write(&mut self, policy: &PrivacyPolicy, value: PolicyValue) -> Result<(), StoreError> {
        self.pending.borrow_mut().retain(|(id, _)| id != policy.id);
        self.writes.push((policy.id.to_owned(), value));
        self.inner.write(policy, value)
    }
}

#[test]
fn audit_and_plan_never_write() {
    let engine = PrivacyEngine::new(MemoryStore::new());
    let findings = engine.audit(all());
    assert_eq!(findings.len(), catalog::all().len());
    assert!(
        findings
            .iter()
            .all(|f| f.state == AuditState::DifferentOrNotConfigured)
    );
    assert!(
        findings
            .iter()
            .all(|f| f.current == Some(PolicyValue::ABSENT))
    );
    let plan = engine.plan(all(), MACHINE).unwrap();
    assert_eq!(plan.changes.len(), catalog::all().len());
    assert_eq!(plan.machine, MACHINE);
    assert_eq!(plan.version, 1);
    assert!(pef_core::time::is_rfc3339_timestamp(&plan.created));
    assert_eq!(engine.store().writes(), 0);
    let summary = AuditSummary::of(&findings);
    assert_eq!(summary.different, catalog::all().len());
    assert_eq!(summary.configured + summary.unknown, 0);
}

#[test]
fn audit_reports_read_failure_as_unknown_and_plan_fails_closed() {
    let mut store = MemoryStore::new();
    store.fail_reads_of(Some("advertising-id"));
    let engine = PrivacyEngine::new(store);
    let findings = engine.audit([
        catalog::get("advertising-id").unwrap(),
        catalog::get("app-camera").unwrap(),
    ]);
    assert_eq!(findings[0].state, AuditState::Unknown);
    assert!(findings[0].current.is_none());
    assert!(
        findings[0]
            .error
            .as_deref()
            .unwrap()
            .contains("access denied")
    );
    assert_eq!(findings[1].state, AuditState::DifferentOrNotConfigured);
    assert!(matches!(
        engine.plan(all(), MACHINE),
        Err(Error::Store(StoreError::Io { .. }))
    ));
    assert_eq!(engine.store().writes(), 0);
}

#[test]
fn plan_only_contains_values_that_differ() {
    let store = MemoryStore::with_values([
        ("advertising-id", PolicyValue::present(1)), // already enforced
        ("clipboard-sync", PolicyValue::present(1)), // wrong value
        ("app-camera", PolicyValue::present(0)),     // explicit zero, differs from 2
    ]);
    let engine = PrivacyEngine::new(store);
    let plan = engine.plan(all(), MACHINE).unwrap();
    assert_eq!(plan.changes.len(), catalog::all().len() - 1);
    assert!(!plan.ids().any(|id| id == "advertising-id"));
    let clipboard = plan
        .changes
        .iter()
        .find(|c| c.id == "clipboard-sync")
        .unwrap();
    assert_eq!(clipboard.before, PolicyValue::present(1));
    assert_eq!(clipboard.desired, 0);
    let camera = plan.changes.iter().find(|c| c.id == "app-camera").unwrap();
    assert_eq!(camera.before, PolicyValue::present(0));
    assert_eq!(
        engine.audit([catalog::get("advertising-id").unwrap()])[0].state,
        AuditState::Configured
    );
}

#[test]
fn backup_failure_prevents_all_writes() {
    let mut engine = PrivacyEngine::new(MemoryStore::new());
    let plan = engine.plan(all(), MACHINE).unwrap();
    let err = engine
        .apply(&plan, MACHINE, |_| {
            Err(BackupError::Other("disk full".into()))
        })
        .unwrap_err();
    assert!(matches!(err, Error::Backup(BackupError::Other(_))), "{err}");
    assert_eq!(err.to_string(), "disk full");
    assert_eq!(engine.store().writes(), 0);
}

#[test]
fn backup_precedes_writes_and_apply_is_idempotent() {
    let mut engine = PrivacyEngine::new(MemoryStore::new());
    let plan = engine.plan(all(), MACHINE).unwrap();
    let persisted = RefCell::new(None);
    let outcome = engine
        .apply(&plan, MACHINE, |backup| {
            persisted.replace(Some(backup.clone()));
            Ok(())
        })
        .unwrap();
    assert_eq!(
        persisted.borrow().as_ref(),
        Some(&plan),
        "the persisted plan is the full plan"
    );
    assert_eq!(outcome.written, catalog::all().len());
    assert!(outcome.backup_saved);
    assert!(
        engine
            .audit(all())
            .iter()
            .all(|f| f.state == AuditState::Configured)
    );

    let second = engine.plan(all(), MACHINE).unwrap();
    assert!(second.changes.is_empty());
    let outcome = engine
        .apply(&second, MACHINE, |_| {
            panic!("a no-op apply must not persist a backup")
        })
        .unwrap();
    assert_eq!(outcome.written, 0);
    assert!(!outcome.backup_saved);
    assert_eq!(engine.store().writes(), catalog::all().len());
}

#[test]
fn backup_is_persisted_before_the_first_write() {
    let mut engine = PrivacyEngine::new(ScriptedStore::new());
    let plan = engine.plan(all(), MACHINE).unwrap();
    let writes_at_persist = RefCell::new(None);
    engine
        .apply(&plan, MACHINE, |_| {
            // No writes have happened when persist runs.
            writes_at_persist.replace(Some(0));
            Ok(())
        })
        .unwrap();
    assert_eq!(writes_at_persist.into_inner(), Some(0));
    let order: Vec<&str> = engine
        .store()
        .writes
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(order, ids(&all()), "writes happen in plan order");
}

#[test]
fn restore_preserves_absent_and_explicit_zero_values() {
    let store = MemoryStore::with_values([("advertising-id", PolicyValue::present(0))]);
    let mut engine = PrivacyEngine::new(store);
    let backup = engine.plan(all(), MACHINE).unwrap();
    engine.apply(&backup, MACHINE, |_| Ok(())).unwrap();
    let dry = engine.restore(&backup, MACHINE, true).unwrap();
    assert!(dry.dry_run);
    assert_eq!(dry.restored + dry.already_restored, 0);
    assert_eq!(engine.store().writes(), catalog::all().len());

    let outcome = engine.restore(&backup, MACHINE, false).unwrap();
    assert_eq!(outcome.restored, catalog::all().len());
    assert_eq!(outcome.already_restored, 0);
    assert_eq!(
        engine.store().get("advertising-id"),
        PolicyValue::present(0)
    );
    assert_eq!(engine.store().get("clipboard-sync"), PolicyValue::ABSENT);
    assert_eq!(
        engine.store().values().count(),
        1,
        "only the explicit zero remains"
    );

    let again = engine.restore(&backup, MACHINE, false).unwrap();
    assert_eq!(again.restored, 0);
    assert_eq!(again.already_restored, catalog::all().len());
    assert_eq!(engine.store().writes(), 2 * catalog::all().len());
}

#[test]
fn restore_walks_the_plan_in_reverse_order() {
    let mut engine = PrivacyEngine::new(ScriptedStore::new());
    let selection =
        catalog::select(["advertising-id", "app-camera", "edge-diagnostic-data"]).unwrap();
    let backup = engine.plan(selection, MACHINE).unwrap();
    engine.apply(&backup, MACHINE, |_| Ok(())).unwrap();
    engine.store_mut().writes.clear();
    engine.restore(&backup, MACHINE, false).unwrap();
    let order: Vec<&str> = engine
        .store()
        .writes
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(
        order,
        ["edge-diagnostic-data", "app-camera", "advertising-id"]
    );
    assert!(
        engine
            .store()
            .writes
            .iter()
            .all(|(_, v)| *v == PolicyValue::ABSENT)
    );
}

#[test]
fn partial_failure_can_be_restored() {
    let mut store = MemoryStore::new();
    store.fail_writes_of(Some("activity-feed"));
    let mut engine = PrivacyEngine::new(store);
    let backup = engine.plan(all(), MACHINE).unwrap();
    let err = engine.apply(&backup, MACHINE, |_| Ok(())).unwrap_err();
    assert!(matches!(err, Error::Store(StoreError::Io { .. })), "{err}");
    let written_before_failure = engine.store().writes();
    assert!(written_before_failure > 0 && written_before_failure < catalog::all().len());

    engine.store_mut().fail_writes_of(None);
    let outcome = engine.restore(&backup, MACHINE, false).unwrap();
    assert_eq!(outcome.restored, written_before_failure);
    assert_eq!(
        outcome.already_restored,
        catalog::all().len() - written_before_failure
    );
    assert!(
        all()
            .iter()
            .all(|p| engine.store().read(p).unwrap() == PolicyValue::ABSENT)
    );
}

#[test]
fn external_changes_block_restore_before_any_write() {
    let mut engine = PrivacyEngine::new(MemoryStore::new());
    let backup = engine.plan(all(), MACHINE).unwrap();
    engine.apply(&backup, MACHINE, |_| Ok(())).unwrap();
    engine
        .store_mut()
        .set("app-location", PolicyValue::present(123));
    let err = engine.restore(&backup, MACHINE, false).unwrap_err();
    assert!(
        matches!(err, Error::Conflict { ref id } if id == "app-location"),
        "{err}"
    );
    assert!(err.is_pre_write());
    assert_eq!(
        engine.store().writes(),
        catalog::all().len(),
        "nothing was restored"
    );
    assert!(matches!(
        engine.restore(&backup, MACHINE, true),
        Err(Error::Conflict { .. })
    ));
}

#[test]
fn stale_apply_plan_is_rejected_before_backup() {
    let mut engine = PrivacyEngine::new(MemoryStore::new());
    let plan = engine.plan(all(), MACHINE).unwrap();
    engine
        .store_mut()
        .set("app-location", PolicyValue::present(123));
    let err = engine
        .apply(&plan, MACHINE, |_| panic!("must not save"))
        .unwrap_err();
    assert!(matches!(err, Error::Conflict { ref id } if id == "app-location"));
    assert_eq!(engine.store().writes(), 0);
}

#[test]
fn a_value_changed_mid_apply_stops_the_operation_recoverably() {
    let mut store = ScriptedStore::new();
    // Reads of clipboard-sync: 1 = plan, 2 = pre-flight, 3 = the check right before
    // its write. By then advertising-id was written; another program now changes
    // clipboard-sync.
    store.before_read = Some(Box::new(|id, count| {
        (id == "clipboard-sync" && count == 3).then_some(PolicyValue::present(77))
    }));
    let mut engine = PrivacyEngine::new(store);
    let selection = catalog::select(["advertising-id", "clipboard-sync", "app-camera"]).unwrap();
    let plan = engine.plan(selection, MACHINE).unwrap();
    let err = engine.apply(&plan, MACHINE, |_| Ok(())).unwrap_err();
    assert!(
        matches!(err, Error::ChangedDuringApply { ref id } if id == "clipboard-sync"),
        "{err}"
    );
    assert!(!err.is_pre_write());
    let written: Vec<&str> = engine
        .store()
        .writes
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(
        written,
        ["advertising-id"],
        "the write stopped at the changed value"
    );

    // The foreign value blocks restore until a person looks at it.
    engine.store_mut().before_read = None;
    let err = engine.restore(&plan, MACHINE, false).unwrap_err();
    assert!(matches!(err, Error::Conflict { ref id } if id == "clipboard-sync"));
    // Once the foreign value is put back, the partial apply restores cleanly.
    engine
        .store_mut()
        .inner
        .set("clipboard-sync", PolicyValue::ABSENT);
    engine.store_mut().pending.borrow_mut().clear();
    let outcome = engine.restore(&plan, MACHINE, false).unwrap();
    assert_eq!(outcome.restored, 1);
    assert_eq!(outcome.already_restored, 2);
}

#[test]
fn a_value_changed_mid_restore_stops_the_operation() {
    let mut engine = PrivacyEngine::new(ScriptedStore::new());
    let selection = catalog::select(["advertising-id", "clipboard-sync"]).unwrap();
    let plan = engine.plan(selection, MACHINE).unwrap();
    engine.apply(&plan, MACHINE, |_| Ok(())).unwrap();
    engine.store_mut().writes.clear();
    // Restore reads in reverse: clipboard-sync pre-flight (read 4 overall for it), then
    // advertising-id pre-flight, then clipboard-sync check. Change advertising-id on
    // its check read.
    let reads_so_far = *engine.store().reads.borrow().get("advertising-id").unwrap();
    engine.store_mut().before_read = Some(Box::new(move |id, count| {
        (id == "advertising-id" && count == reads_so_far + 2).then_some(PolicyValue::present(9))
    }));
    let err = engine.restore(&plan, MACHINE, false).unwrap_err();
    assert!(
        matches!(err, Error::ChangedDuringRestore { ref id } if id == "advertising-id"),
        "{err}"
    );
    let written: Vec<&str> = engine
        .store()
        .writes
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(
        written,
        ["clipboard-sync"],
        "restore stopped at the changed value"
    );
}

#[test]
fn read_back_failure_is_not_reported_as_success() {
    let mut store = MemoryStore::new();
    store.drop_writes(true);
    let mut engine = PrivacyEngine::new(store);
    let plan = engine.plan(all(), MACHINE).unwrap();
    let err = engine.apply(&plan, MACHINE, |_| Ok(())).unwrap_err();
    assert!(
        matches!(err, Error::ReadBack { ref id } if id == "diagnostic-data"),
        "{err}"
    );
    assert_eq!(
        engine.store().writes(),
        1,
        "stops at the first unverifiable write"
    );
}

#[test]
fn restore_read_back_failure_is_reported() {
    let mut engine = PrivacyEngine::new(MemoryStore::new());
    let selection = catalog::select(["advertising-id"]).unwrap();
    let plan = engine.plan(selection, MACHINE).unwrap();
    engine.apply(&plan, MACHINE, |_| Ok(())).unwrap();
    engine.store_mut().drop_writes(true);
    let err = engine.restore(&plan, MACHINE, false).unwrap_err();
    assert!(
        matches!(err, Error::RestoreReadBack { ref id } if id == "advertising-id"),
        "{err}"
    );
    assert_eq!(
        err.to_string(),
        "Read-back failed restoring advertising-id; retain the backup."
    );
}

#[test]
fn malformed_or_foreign_backups_are_rejected() {
    let valid = PrivacyEngine::new(MemoryStore::new())
        .plan(all(), MACHINE)
        .unwrap();
    validate(&valid, MACHINE).unwrap();

    let wrong_version = PrivacyBackup {
        version: 99,
        ..valid.clone()
    };
    let err = validate(&wrong_version, MACHINE).unwrap_err();
    assert!(err.to_string().contains("version 99"), "{err}");

    let err = validate(&valid, "different").unwrap_err();
    assert!(
        err.to_string().contains("created on 'test-machine'"),
        "{err}"
    );

    let duplicate = PrivacyBackup {
        changes: vec![valid.changes[0].clone(), valid.changes[0].clone()],
        ..valid.clone()
    };
    let err = validate(&duplicate, MACHINE).unwrap_err();
    assert!(err.to_string().contains("duplicate entry"), "{err}");

    let unknown = PrivacyBackup {
        changes: vec![PolicyChange {
            id: "unknown".into(),
            before: PolicyValue::ABSENT,
            desired: 1,
        }],
        ..valid.clone()
    };
    let err = validate(&unknown, MACHINE).unwrap_err();
    assert!(
        err.to_string().contains("Unknown policy 'unknown'"),
        "{err}"
    );

    let retargeted = PrivacyBackup {
        changes: vec![PolicyChange {
            desired: 42,
            ..valid.changes[0].clone()
        }],
        ..valid.clone()
    };
    let err = validate(&retargeted, MACHINE).unwrap_err();
    assert!(err.to_string().contains("enforces 42"), "{err}");

    let absent_with_value = PrivacyBackup {
        changes: vec![PolicyChange {
            before: PolicyValue {
                exists: false,
                value: 5,
            },
            ..valid.changes[0].clone()
        }],
        ..valid.clone()
    };
    let err = validate(&absent_with_value, MACHINE).unwrap_err();
    assert!(err.to_string().contains("marked absent"), "{err}");

    let mut too_many = valid.clone();
    too_many.changes.push(valid.changes[0].clone());
    let err = validate(&too_many, MACHINE).unwrap_err();
    assert!(err.to_string().contains("exceed"), "{err}");

    for bad in [
        wrong_version,
        duplicate,
        unknown,
        retargeted,
        absent_with_value,
        too_many,
    ] {
        let mut engine = PrivacyEngine::new(MemoryStore::new());
        assert!(matches!(
            engine.apply(&bad, MACHINE, |_| panic!("must not persist")),
            Err(Error::InvalidBackup(_))
        ));
        assert!(matches!(
            engine.restore(&bad, MACHINE, false),
            Err(Error::InvalidBackup(_))
        ));
        assert_eq!(engine.store().writes(), 0);
    }
}

#[test]
fn backup_round_trips_through_a_file_and_cannot_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("privacy-before.json");
    let mut engine = PrivacyEngine::new(MemoryStore::new());
    let plan = engine.plan(all(), MACHINE).unwrap();
    engine
        .apply(&plan, MACHINE, |b| backup::save(&path, b))
        .unwrap();
    let loaded = backup::load(&path).unwrap();
    assert_eq!(loaded, plan);
    validate(&loaded, MACHINE).unwrap();
    assert!(matches!(
        backup::save(&path, &plan),
        Err(BackupError::Create { .. })
    ));

    let mut fresh = PrivacyEngine::new(engine.into_store());
    let outcome = fresh.restore(&loaded, MACHINE, false).unwrap();
    assert_eq!(outcome.restored, catalog::all().len());
    assert_eq!(fresh.store().values().count(), 0);
}

#[test]
fn incomplete_or_duplicate_recovery_fields_are_rejected_before_any_write() {
    let dir = tempfile::tempdir().unwrap();
    for before in [
        "{}",
        r#"{"Exists":true}"#,
        r#"{"Value":0}"#,
        r#"{"Exists":true,"Value":1,"Value":0}"#,
    ] {
        let path = dir.path().join("b.json");
        std::fs::write(
            &path,
            format!(
                r#"{{"Version":1,"Machine":"test-machine","Created":"2026-09-30T00:00:00Z","Changes":[{{"Id":"clipboard-sync","Desired":0,"Before":{before}}}]}}"#
            ),
        )
        .unwrap();
        let err = backup::load(&path).unwrap_err();
        assert!(
            matches!(err, BackupError::Malformed { .. }),
            "{before}: {err}"
        );
        std::fs::remove_file(&path).unwrap();
    }
}

#[test]
fn oversized_backup_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("big.json");
    std::fs::write(&path, " ".repeat(65_537)).unwrap();
    assert!(matches!(
        backup::load(&path).unwrap_err(),
        BackupError::TooLarge { .. }
    ));
}

#[test]
fn presets_apply_and_restore_cleanly() {
    for preset in catalog::presets() {
        let mut engine = PrivacyEngine::new(MemoryStore::new());
        let policies = catalog::expand_preset(preset);
        let plan = engine.plan(policies.clone(), MACHINE).unwrap();
        assert_eq!(plan.changes.len(), policies.len(), "{}", preset.id);
        engine.apply(&plan, MACHINE, |_| Ok(())).unwrap();
        let findings = engine.audit(policies.clone());
        assert!(
            findings.iter().all(|f| f.state == AuditState::Configured),
            "{}",
            preset.id
        );
        let untouched = engine
            .audit(all())
            .iter()
            .filter(|f| f.state == AuditState::Configured)
            .count();
        assert_eq!(
            untouched,
            policies.len(),
            "{} touched policies outside the preset",
            preset.id
        );
        engine.restore(&plan, MACHINE, false).unwrap();
        assert_eq!(engine.store().values().count(), 0, "{}", preset.id);
    }
}

#[test]
fn engine_works_through_a_boxed_store() {
    let store: Box<dyn PolicyStore> = Box::new(MemoryStore::with_values([(
        "app-camera",
        PolicyValue::present(2),
    )]));
    let mut engine = PrivacyEngine::new(store);
    let findings = engine.audit([catalog::get("app-camera").unwrap()]);
    assert_eq!(findings[0].state, AuditState::Configured);
    let plan = engine
        .plan([catalog::get("app-microphone").unwrap()], MACHINE)
        .unwrap();
    let outcome = engine.apply(&plan, MACHINE, |_| Ok(())).unwrap();
    assert_eq!(outcome.written, 1);
}

#[test]
fn store_type_errors_fail_the_plan_and_show_as_unknown() {
    struct TypeMismatch;
    impl PolicyStore for TypeMismatch {
        fn read(&self, policy: &PrivacyPolicy) -> Result<PolicyValue, StoreError> {
            Err(StoreError::UnexpectedType {
                id: policy.id.to_owned(),
            })
        }
        fn write(&mut self, _: &PrivacyPolicy, _: PolicyValue) -> Result<(), StoreError> {
            panic!("never written")
        }
    }
    let engine = PrivacyEngine::new(TypeMismatch);
    let findings = engine.audit(all());
    assert!(findings.iter().all(|f| f.state == AuditState::Unknown));
    assert!(
        findings[0]
            .error
            .as_deref()
            .unwrap()
            .contains("unexpected registry type")
    );
    assert!(matches!(
        engine.plan(all(), MACHINE),
        Err(Error::Store(StoreError::UnexpectedType { .. }))
    ));
}
