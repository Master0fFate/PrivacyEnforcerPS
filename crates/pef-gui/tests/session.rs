//! The session model, exercised without any user-interface code.

use pef_core::{AuditState, Category, MemoryStore, PolicyValue, catalog};
use pef_gui::session::{NoticeKind, Session, SystemStatus};

fn session_with(store: MemoryStore, elevated: bool) -> Session {
    Session::new(
        Box::new(store),
        SystemStatus {
            description: "Windows 11 Professional 24H2 (build 26100)".into(),
            elevated,
            simulated: false,
            machine: "TEST-PC".into(),
        },
    )
}

fn value(session: &Session, id: &str) -> PolicyValue {
    session.store().read(catalog::get(id).unwrap()).unwrap()
}

#[test]
fn simulated_session_is_labelled_and_writable() {
    let mut session = Session::simulated("No registry here", "DEV-BOX");
    assert!(session.status().simulated);
    assert!(!session.status().elevated);
    assert!(session.can_write(), "simulation stands in for elevation");
    assert_eq!(session.status().machine, "DEV-BOX");
    assert!(!session.audited());
    assert!(session.findings().is_empty());
    assert_eq!(session.summary().total(), 0);

    session.run_audit();
    assert!(session.audited());
    assert_eq!(session.findings().len(), catalog::all().len());
    assert_eq!(
        session.finding("advertising-id").unwrap().state,
        AuditState::Configured
    );
    assert_eq!(
        session.finding("clipboard-sync").unwrap().state,
        AuditState::DifferentOrNotConfigured
    );
    assert_eq!(session.summary().configured, 1);
    let notice = session.notice().unwrap();
    assert_eq!(notice.kind, NoticeKind::Info);
    assert!(notice.title.starts_with("1 of "));
}

#[test]
fn audit_reports_unknown_values_as_a_warning() {
    let mut store = MemoryStore::new();
    store.fail_reads_of(Some("app-camera"));
    let mut session = session_with(store, false);
    session.run_audit();
    assert_eq!(session.summary().unknown, 1);
    assert_eq!(session.notice().unwrap().kind, NoticeKind::Warning);
    assert!(session.finding("app-camera").unwrap().error.is_some());
}

#[test]
fn selection_is_explicit_and_kept_in_catalog_order() {
    let mut session = Session::simulated("x", "PC");
    assert_eq!(session.selected_count(), 0);
    session.toggle("app-camera");
    session.toggle("advertising-id");
    session.set_selected("unknown-policy", true);
    assert_eq!(session.selected_count(), 2);
    let ids: Vec<&str> = session.selection().iter().map(|p| p.id).collect();
    assert_eq!(ids, ["advertising-id", "app-camera"]);
    session.toggle("app-camera");
    assert!(!session.is_selected("app-camera"));
    session.set_selected("app-camera", false);
    assert_eq!(session.selected_count(), 1);

    assert_eq!(session.select_preset("edge").unwrap(), 4);
    assert_eq!(session.selected_count(), 5);
    assert!(session.category_fully_selected(Category::Edge));
    assert!(!session.category_fully_selected(Category::Search));
    session.set_category_selected(Category::Search, true);
    assert!(session.category_fully_selected(Category::Search));
    session.set_category_selected(Category::Edge, false);
    assert!(!session.is_selected("edge-diagnostic-data"));
    assert!(session.select_preset("nope").is_err());

    session.clear_selection();
    assert_eq!(session.selected_count(), 0);
}

#[test]
fn preview_needs_a_selection_and_is_discarded_when_it_changes() {
    let mut session = Session::simulated("x", "PC");
    assert_eq!(
        session.preview_enforce().unwrap_err(),
        "Select at least one policy first."
    );
    assert!(session.plan().is_none());
    session.toggle("advertising-id"); // already configured in the simulation
    session.toggle("app-camera");
    let plan = session.preview_enforce().unwrap().clone();
    assert_eq!(plan.changes.len(), 1, "only values that differ are planned");
    assert_eq!(plan.changes[0].id, "app-camera");
    assert_eq!(plan.changes[0].before, PolicyValue::present(0));
    assert!(session.plan().is_some());
    session.toggle("clipboard-sync");
    assert!(
        session.plan().is_none(),
        "a changed selection invalidates the preview"
    );
    session.select_preset("edge").unwrap();
    assert!(session.plan().is_none());
}

#[test]
fn apply_blockers_are_explained() {
    let mut not_elevated = session_with(MemoryStore::new(), false);
    assert!(!not_elevated.can_write());
    not_elevated.toggle("advertising-id");
    not_elevated.preview_enforce().unwrap();
    assert!(
        not_elevated
            .apply_blocker()
            .unwrap()
            .starts_with("Not elevated")
    );
    assert!(
        not_elevated
            .restore_blocker()
            .unwrap()
            .starts_with("Not elevated")
    );

    let mut elevated = session_with(MemoryStore::new(), true);
    assert_eq!(
        elevated.apply_blocker().as_deref(),
        Some("Preview the selection first.")
    );
    assert_eq!(
        elevated.restore_blocker().as_deref(),
        Some("Open and validate a backup file first.")
    );
    elevated.toggle("advertising-id");
    elevated.preview_enforce().unwrap();
    assert!(elevated.apply_blocker().is_none());

    let mut configured = session_with(
        MemoryStore::with_values([("advertising-id", PolicyValue::present(1))]),
        true,
    );
    configured.toggle("advertising-id");
    configured.preview_enforce().unwrap();
    assert!(
        configured
            .apply_blocker()
            .unwrap()
            .starts_with("Nothing to change")
    );
}

#[test]
fn apply_needs_the_exact_word_and_a_backup_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("b.json");
    let mut session = session_with(MemoryStore::new(), true);
    session.toggle("advertising-id");
    session.preview_enforce().unwrap();
    for wrong in ["enforce", "ENFORCE ", "", "RESTORE"] {
        let err = session.apply_enforce(&path, wrong).unwrap_err();
        assert_eq!(
            err, "Type ENFORCE exactly to confirm. Nothing was written.",
            "{wrong:?}"
        );
    }
    let err = session
        .apply_enforce(std::path::Path::new(""), "ENFORCE")
        .unwrap_err();
    assert!(err.contains("Choose a new backup file"));
    assert_eq!(value(&session, "advertising-id"), PolicyValue::ABSENT);
    assert!(!path.exists());
    assert!(
        session.plan().is_some(),
        "a rejected confirmation keeps the preview"
    );
}

#[test]
fn enforce_then_restore_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("privacy-before.json");
    let store = MemoryStore::with_values([("clipboard-sync", PolicyValue::present(1))]);
    let mut session = session_with(store, true);
    session.toggle("advertising-id");
    session.toggle("clipboard-sync");
    session.preview_enforce().unwrap();

    let outcome = session.apply_enforce(&path, "ENFORCE").unwrap();
    assert_eq!(outcome.written, 2);
    assert!(outcome.backup_saved);
    assert!(path.exists());
    assert_eq!(value(&session, "advertising-id"), PolicyValue::present(1));
    assert_eq!(value(&session, "clipboard-sync"), PolicyValue::present(0));
    assert!(session.plan().is_none(), "the applied plan is consumed");
    assert!(session.audited(), "an apply re-audits");
    assert_eq!(
        session.finding("advertising-id").unwrap().state,
        AuditState::Configured
    );
    let notice = session.notice().unwrap();
    assert_eq!(notice.kind, NoticeKind::Success);
    assert!(notice.title.starts_with("2 value(s) written"));
    assert!(notice.detail.contains("privacy-before.json"));

    // A second preview of the same selection has nothing to do.
    session.preview_enforce().unwrap();
    assert!(
        session
            .apply_blocker()
            .unwrap()
            .starts_with("Nothing to change")
    );

    let loaded = session.load_backup(&path).unwrap().clone();
    assert_eq!(loaded.path, path);
    assert_eq!(loaded.backup.changes.len(), 2);
    assert!(session.restore_blocker().is_none());
    assert_eq!(
        session.apply_restore("restore").unwrap_err(),
        "Type RESTORE exactly to confirm. Nothing was written."
    );
    let outcome = session.apply_restore("RESTORE").unwrap();
    assert_eq!(outcome.restored, 2);
    assert_eq!(outcome.already_restored, 0);
    assert_eq!(value(&session, "advertising-id"), PolicyValue::ABSENT);
    assert_eq!(value(&session, "clipboard-sync"), PolicyValue::present(1));
    assert_eq!(session.notice().unwrap().kind, NoticeKind::Success);

    let outcome = session.apply_restore("RESTORE").unwrap();
    assert_eq!(outcome.restored, 0);
    assert_eq!(outcome.already_restored, 2);
}

#[test]
fn load_backup_rejects_missing_foreign_and_conflicting_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut session = session_with(MemoryStore::new(), true);
    let err = session
        .load_backup(&dir.path().join("missing.json"))
        .unwrap_err();
    assert!(err.contains("could not be opened"), "{err}");
    assert!(session.loaded_backup().is_none());

    let foreign = dir.path().join("foreign.json");
    std::fs::write(
        &foreign,
        r#"{"Version":1,"Machine":"OTHER","Created":"2026-01-01T00:00:00Z","Changes":[]}"#,
    )
    .unwrap();
    let err = session.load_backup(&foreign).unwrap_err();
    assert!(err.contains("created on 'OTHER'"), "{err}");

    let conflicting = dir.path().join("conflict.json");
    std::fs::write(&conflicting, r#"{"Version":1,"Machine":"TEST-PC","Created":"2026-01-01T00:00:00Z","Changes":[{"Id":"app-camera","Before":{"Exists":false,"Value":0},"Desired":2}]}"#).unwrap();
    assert!(
        session.load_backup(&conflicting).is_ok(),
        "absent matches the recorded original"
    );
    let mut changed = session_with(
        MemoryStore::with_values([("app-camera", PolicyValue::present(7))]),
        true,
    );
    let err = changed.load_backup(&conflicting).unwrap_err();
    assert!(err.starts_with("Conflict for app-camera"), "{err}");
    assert!(changed.loaded_backup().is_none());
}

#[test]
fn failures_produce_error_notices_with_recovery_guidance() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join("existing.json");
    std::fs::write(&existing, "keep").unwrap();
    let mut session = session_with(MemoryStore::new(), true);
    session.toggle("advertising-id");
    session.preview_enforce().unwrap();
    let err = session.apply_enforce(&existing, "ENFORCE").unwrap_err();
    assert!(err.contains("could not be created"), "{err}");
    let notice = session.notice().unwrap();
    assert_eq!(notice.kind, NoticeKind::Error);
    assert_eq!(notice.detail, "No policy values were written.");
    assert_eq!(value(&session, "advertising-id"), PolicyValue::ABSENT);
    assert_eq!(std::fs::read_to_string(&existing).unwrap(), "keep");

    let mut store = MemoryStore::new();
    store.fail_writes_of(Some("app-camera"));
    let mut session = session_with(store, true);
    session.toggle("advertising-id");
    session.toggle("app-camera");
    session.preview_enforce().unwrap();
    let path = dir.path().join("partial.json");
    session.apply_enforce(&path, "ENFORCE").unwrap_err();
    let notice = session.notice().unwrap();
    assert_eq!(notice.kind, NoticeKind::Error);
    assert!(
        notice.detail.contains("began before it stopped"),
        "{}",
        notice.detail
    );
    assert!(
        path.exists(),
        "the recovery file was saved before the first write"
    );
    assert_eq!(value(&session, "advertising-id"), PolicyValue::present(1));
}

#[test]
fn notices_can_be_set_and_cleared() {
    let mut session = Session::simulated("x", "PC");
    assert!(session.notice().is_none());
    session.set_notice(NoticeKind::Warning, "Heads up", "Detail");
    assert_eq!(session.notice().unwrap().title, "Heads up");
    session.clear_notice();
    assert!(session.notice().is_none());
}
