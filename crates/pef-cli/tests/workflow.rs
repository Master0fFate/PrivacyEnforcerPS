//! In-process tests of the full command line workflow with a fake host.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::rc::Rc;

use pef_cli::{EXIT_OK, EXIT_UNKNOWN_FINDINGS, Failure, Host, run};
use pef_core::{MemoryStore, PolicyStore, PolicyValue, PrivacyPolicy, StoreError, backup, catalog};

#[derive(Clone)]
struct SharedStore(Rc<RefCell<MemoryStore>>);

impl PolicyStore for SharedStore {
    fn read(&self, policy: &PrivacyPolicy) -> Result<PolicyValue, StoreError> {
        self.0.borrow().read(policy)
    }
    fn write(&mut self, policy: &PrivacyPolicy, value: PolicyValue) -> Result<(), StoreError> {
        self.0.borrow_mut().write(policy, value)
    }
}

struct FakeHost {
    store: Rc<RefCell<MemoryStore>>,
    gate: Result<String, String>,
    elevated: bool,
    terminal: bool,
    answers: VecDeque<String>,
}

impl FakeHost {
    fn supported() -> Self {
        FakeHost {
            store: Rc::new(RefCell::new(MemoryStore::new())),
            gate: Ok("Windows 11 Professional 24H2 (build 26100)".into()),
            elevated: true,
            terminal: true,
            answers: VecDeque::new(),
        }
    }

    fn answering(mut self, answer: &str) -> Self {
        self.answers.push_back(answer.to_owned());
        self
    }

    fn writes(&self) -> usize {
        self.store.borrow().writes()
    }

    fn value(&self, id: &str) -> PolicyValue {
        self.store.borrow().get(id)
    }
}

impl Host for FakeHost {
    fn open_store(&mut self) -> Result<(String, Box<dyn PolicyStore>), String> {
        let system = self.gate.clone()?;
        Ok((system, Box::new(SharedStore(self.store.clone()))))
    }
    fn machine_name(&self) -> String {
        "TEST-PC".into()
    }
    fn is_elevated(&self) -> bool {
        self.elevated
    }
    fn input_is_terminal(&self) -> bool {
        self.terminal
    }
    fn read_line(&mut self) -> io::Result<Option<String>> {
        Ok(self.answers.pop_front())
    }
}

fn exec(host: &mut FakeHost, line: &str) -> (Result<u8, Failure>, String) {
    let args: Vec<String> = line
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let mut out = Vec::new();
    let result = run(&args, host, &mut out);
    (result, String::from_utf8(out).unwrap())
}

#[test]
fn help_and_version_need_no_platform() {
    let mut host = FakeHost::supported();
    host.gate = Err("no platform".into());
    let (code, out) = exec(&mut host, "help");
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.contains("enforce (--select ID,ID | --preset NAME,NAME)"));
    let (code, out) = exec(&mut host, "version");
    assert_eq!(code, Ok(EXIT_OK));
    assert_eq!(out.trim(), format!("pef {}", pef_core::VERSION));
}

#[test]
fn list_works_without_platform_access() {
    let mut host = FakeHost::supported();
    host.gate = Err("no platform".into());
    let (code, out) = exec(&mut host, "list");
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.contains("== Diagnostic data:"));
    assert!(out.contains("advertising-id: HKLM\\SOFTWARE\\Policies\\Microsoft\\Windows\\AdvertisingInfo\\DisabledByGroupPolicy = 1"));
    assert!(out.contains("Source: https://learn.microsoft.com/"));
    assert_eq!(out.matches("\n  Impact: ").count(), catalog::all().len());

    let (code, out) = exec(&mut host, "list --select app-camera,advertising-id");
    assert_eq!(code, Ok(EXIT_OK));
    assert_eq!(out.matches("\n  Impact: ").count(), 2);
    assert!(
        out.find("advertising-id:").unwrap() < out.find("app-camera:").unwrap(),
        "catalog order"
    );

    let (code, out) = exec(&mut host, "list --json");
    assert_eq!(code, Ok(EXIT_OK));
    let json: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(json.len(), catalog::all().len());
    assert_eq!(json[0]["Category"], "DiagnosticData");

    let (code, out) = exec(&mut host, "list --presets");
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.contains("baseline: Baseline ("));
    assert!(out.contains("Policies: "));
    let (code, out) = exec(&mut host, "list --presets --json");
    assert_eq!(code, Ok(EXIT_OK));
    let json: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(json.len(), 4);
    assert_eq!(
        json[3]["Policies"].as_array().unwrap().len(),
        catalog::all().len()
    );
    assert_eq!(host.writes(), 0);
}

#[test]
fn invalid_input_stops_before_the_platform_gate() {
    let mut host = FakeHost::supported();
    host.gate = Err("must not be consulted".into());
    for line in [
        "enforce --select deliberately-invalid-policy",
        "enforce",
        "audit --apply",
        "restore",
        "nonsense",
    ] {
        let (code, out) = exec(&mut host, line);
        let failure = code.unwrap_err();
        assert!(!failure.may_have_written, "{line}");
        assert!(failure.render().starts_with("Stopped: "), "{line}");
        assert!(
            failure.render().ends_with("No policy values were written."),
            "{line}"
        );
        assert!(out.is_empty(), "{line}: {out}");
    }
    let (code, _) = exec(&mut host, "enforce --select deliberately-invalid-policy");
    assert_eq!(
        code.unwrap_err().message,
        "Unknown policy 'deliberately-invalid-policy'. Run list for valid IDs."
    );
}

#[test]
fn unsupported_platform_stops_audit_enforce_and_restore() {
    let mut host = FakeHost::supported();
    host.gate = Err("Requires Windows 11".into());
    for line in [
        "",
        "audit",
        "enforce --select advertising-id",
        "restore --backup x.json",
    ] {
        let (code, out) = exec(&mut host, line);
        assert_eq!(code.unwrap_err().message, "Requires Windows 11", "{line}");
        assert!(out.is_empty());
    }
    assert_eq!(host.writes(), 0);
}

#[test]
fn audit_is_read_only_and_reports_unknowns_with_exit_2() {
    let mut host = FakeHost::supported();
    host.store
        .borrow_mut()
        .set("advertising-id", PolicyValue::present(1));
    let (code, out) = exec(&mut host, "");
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.starts_with(
        "Read-only policy registry audit: Windows 11 Professional 24H2 (build 26100)."
    ));
    assert!(out.contains("advertising-id: Configured; current=1, desired=1"));
    assert!(out.contains(
        "app-camera: DifferentOrNotConfigured; current=not configured/unknown, desired=2"
    ));
    assert!(out.contains(&format!(
        "Summary: 1 configured, {} different or not configured, 0 unknown ({} policies).",
        catalog::all().len() - 1,
        catalog::all().len()
    )));

    host.store.borrow_mut().fail_reads_of(Some("app-camera"));
    let (code, out) = exec(&mut host, "audit --select app-camera,advertising-id");
    assert_eq!(code, Ok(EXIT_UNKNOWN_FINDINGS));
    assert!(out.contains(
        "app-camera: Unknown; current=not configured/unknown, desired=2\n  Denies camera"
    ));
    assert!(out.contains("  Error: reading app-camera: access denied (injected)"));

    let (code, out) = exec(&mut host, "audit --select app-camera --json");
    assert_eq!(code, Ok(EXIT_UNKNOWN_FINDINGS));
    let json: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(json["Summary"]["Unknown"], 1);
    assert_eq!(json["Findings"][0]["State"], "Unknown");
    assert_eq!(json["System"], "Windows 11 Professional 24H2 (build 26100)");
    assert_eq!(host.writes(), 0);
}

#[test]
fn enforce_previews_by_default_and_never_prompts() {
    let mut host = FakeHost::supported();
    let (code, out) = exec(&mut host, "enforce --select advertising-id,clipboard-sync");
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.starts_with("DRY RUN: enforce; 2 selected changes\n"));
    assert!(out.contains("advertising-id: before=absent, enforced=1\n  Advertising ID (Moderate impact)\n  Stops apps using"));
    assert!(!out.contains("Type ENFORCE"));
    assert_eq!(host.writes(), 0);

    let (code, out) = exec(&mut host, "enforce --preset edge");
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.starts_with("DRY RUN: enforce; 4 selected changes\n"));
}

#[test]
fn apply_requires_elevation_and_a_real_terminal() {
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("b.json");
    let line = format!(
        "enforce --select advertising-id --apply --backup {}",
        backup.display()
    );

    let mut host = FakeHost::supported();
    host.elevated = false;
    let (code, out) = exec(&mut host, &line);
    assert_eq!(
        code.unwrap_err().message,
        "Apply/restore requires an elevated terminal. Audit and dry-run do not."
    );
    assert!(out.starts_with("Requested operation: enforce; 1 selected changes"));

    let mut host = FakeHost::supported();
    host.terminal = false;
    let (code, _) = exec(&mut host, &line);
    assert!(
        code.unwrap_err()
            .message
            .starts_with("Interactive confirmation required.")
    );
    assert!(!backup.exists());
    assert_eq!(host.writes(), 0);
}

#[test]
fn apply_needs_the_exact_confirmation_word() {
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("b.json");
    let line = format!(
        "enforce --select advertising-id --apply --backup {}",
        backup.display()
    );
    for wrong in ["enforce", "ENFORCE ", "yes", "RESTORE", ""] {
        let mut host = FakeHost::supported().answering(wrong);
        let (code, out) = exec(&mut host, &line);
        assert_eq!(code, Ok(EXIT_OK), "{wrong:?}");
        assert!(
            out.ends_with(
                "Type ENFORCE to confirm only these policies: Cancelled. No policy writes.\n"
            ),
            "{wrong:?}: {out}"
        );
        assert_eq!(host.writes(), 0);
        assert!(!backup.exists());
    }
    let mut host = FakeHost::supported();
    let (code, out) = exec(&mut host, &line);
    assert_eq!(code, Ok(EXIT_OK), "end of input cancels");
    assert!(out.contains("Cancelled."));
}

#[test]
fn apply_writes_a_backup_then_the_registry_and_restore_reverts() {
    let dir = tempfile::tempdir().unwrap();
    let backup_path = dir.path().join("privacy-before.json");
    let mut host = FakeHost::supported().answering("ENFORCE");
    host.store
        .borrow_mut()
        .set("clipboard-sync", PolicyValue::present(1));
    let line = format!(
        "enforce --select advertising-id,clipboard-sync,app-camera --apply --backup {}",
        backup_path.display()
    );
    let (code, out) = exec(&mut host, &line);
    assert_eq!(code, Ok(EXIT_OK), "{out}");
    assert!(out.contains("These are machine-wide policies."));
    assert!(out.contains(&format!("Recovery file: {}.", backup_path.display())));
    assert!(out.contains("Registry values verified: 3 written."));
    assert_eq!(host.writes(), 3);
    assert_eq!(host.value("advertising-id"), PolicyValue::present(1));
    assert_eq!(host.value("clipboard-sync"), PolicyValue::present(0));
    assert_eq!(host.value("app-camera"), PolicyValue::present(2));

    let saved = backup::load(&backup_path).unwrap();
    assert_eq!(saved.machine, "TEST-PC");
    assert_eq!(saved.changes.len(), 3);
    let clipboard = saved
        .changes
        .iter()
        .find(|c| c.id == "clipboard-sync")
        .unwrap();
    assert_eq!(clipboard.before, PolicyValue::present(1));
    let order: Vec<&str> = saved.changes.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        order,
        ["advertising-id", "app-camera", "clipboard-sync"],
        "catalog order"
    );

    // Repeating the same enforcement is a no-op and must not need a new backup file.
    let mut host2 = FakeHost {
        store: host.store.clone(),
        ..FakeHost::supported()
    };
    let (code, out) = exec(&mut host2, &line);
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.contains("0 selected changes"));
    assert!(out.contains("Nothing to change"));
    assert_eq!(host.writes(), 3);

    // Restore previews first, then needs RESTORE.
    let restore_line = format!("restore --backup {}", backup_path.display());
    let mut host3 = FakeHost {
        store: host.store.clone(),
        ..FakeHost::supported()
    };
    let (code, out) = exec(&mut host3, &restore_line);
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.starts_with("DRY RUN: restore; 3 selected changes"));
    assert_eq!(host.writes(), 3);

    let mut host4 = FakeHost {
        store: host.store.clone(),
        ..FakeHost::supported()
    }
    .answering("RESTORE");
    let (code, out) = exec(&mut host4, &format!("{restore_line} --apply"));
    assert_eq!(code, Ok(EXIT_OK), "{out}");
    assert!(
        out.contains("Registry values verified: 3 restored, 0 already held their original value.")
    );
    assert_eq!(host.value("advertising-id"), PolicyValue::ABSENT);
    assert_eq!(host.value("clipboard-sync"), PolicyValue::present(1));
    assert_eq!(host.value("app-camera"), PolicyValue::ABSENT);

    let mut host5 = FakeHost {
        store: host.store.clone(),
        ..FakeHost::supported()
    }
    .answering("RESTORE");
    let (code, out) = exec(&mut host5, &format!("{restore_line} --apply"));
    assert_eq!(code, Ok(EXIT_OK));
    assert!(out.contains("0 restored, 3 already held their original value."));
}

#[test]
fn an_existing_backup_file_blocks_apply_before_any_write() {
    let dir = tempfile::tempdir().unwrap();
    let backup_path = dir.path().join("exists.json");
    std::fs::write(&backup_path, "keep me").unwrap();
    let mut host = FakeHost::supported().answering("ENFORCE");
    let line = format!(
        "enforce --select advertising-id --apply --backup {}",
        backup_path.display()
    );
    let (code, _) = exec(&mut host, &line);
    let failure = code.unwrap_err();
    assert!(
        failure.message.contains("could not be created"),
        "{}",
        failure.message
    );
    assert!(!failure.may_have_written);
    assert_eq!(host.writes(), 0);
    assert_eq!(std::fs::read_to_string(&backup_path).unwrap(), "keep me");
}

#[test]
fn restore_rejects_foreign_malformed_and_conflicting_backups() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("b.json");

    std::fs::write(
        &path,
        r#"{"Version":1,"Machine":"OTHER-PC","Created":"2026-01-01T00:00:00Z","Changes":[]}"#,
    )
    .unwrap();
    let mut host = FakeHost::supported();
    let (code, _) = exec(&mut host, &format!("restore --backup {}", path.display()));
    assert!(code.unwrap_err().message.contains("created on 'OTHER-PC'"));

    std::fs::write(&path, r#"{"Version":1,"Machine":"TEST-PC","Created":"2026-01-01T00:00:00Z","Changes":[{"Id":"app-camera","Before":{"Exists":false,"Value":0},"Desired":2}]}"#).unwrap();
    host.store
        .borrow_mut()
        .set("app-camera", PolicyValue::present(1));
    let (code, _) = exec(&mut host, &format!("restore --backup {}", path.display()));
    let failure = code.unwrap_err();
    assert!(
        failure.message.starts_with("Conflict for app-camera"),
        "{}",
        failure.message
    );
    assert!(!failure.may_have_written);

    std::fs::write(&path, "{not json").unwrap();
    let (code, _) = exec(&mut host, &format!("restore --backup {}", path.display()));
    assert!(
        code.unwrap_err()
            .message
            .contains("not a valid Privacy Enforcer backup")
    );

    let (code, _) = exec(
        &mut host,
        &format!(
            "restore --backup {}",
            dir.path().join("missing.json").display()
        ),
    );
    assert!(code.unwrap_err().message.contains("could not be opened"));
    assert_eq!(host.writes(), 0);
}

#[test]
fn a_write_failure_mid_apply_is_reported_as_possibly_written() {
    let dir = tempfile::tempdir().unwrap();
    let backup_path = dir.path().join("b.json");
    let mut host = FakeHost::supported().answering("ENFORCE");
    host.store
        .borrow_mut()
        .fail_writes_of(Some("clipboard-sync"));
    let line = format!(
        "enforce --select advertising-id,clipboard-sync --apply --backup {}",
        backup_path.display()
    );
    let (code, _) = exec(&mut host, &line);
    let failure = code.unwrap_err();
    assert!(failure.may_have_written);
    assert!(failure.render().contains("Retain the backup"));
    assert_eq!(host.writes(), 1);
    assert!(
        backup_path.exists(),
        "the backup was saved before the first write"
    );
    let saved = backup::load(&backup_path).unwrap();
    assert_eq!(saved.changes.len(), 2);
}
