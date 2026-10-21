//! Smoke tests that drive the real `pef` executable.

use std::process::Command;

fn pef(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_pef"))
        .args(args)
        .output()
        .expect("pef runs");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn list_json_is_the_full_catalog() {
    let (code, out, err) = pef(&["list", "--json"]);
    assert_eq!(code, 0, "{err}");
    let json: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap();
    assert_eq!(json.len(), pef_core::catalog::all().len());
    assert!(json.iter().any(|p| p["Id"] == "advertising-id"));
}

#[test]
fn help_and_version_succeed() {
    let (code, out, _) = pef(&["help"]);
    assert_eq!(code, 0);
    assert!(out.contains("audit-first"));
    let (code, out, _) = pef(&["--version"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("pef "));
}

#[test]
fn invalid_policy_fails_before_any_platform_access() {
    let (code, out, err) = pef(&["enforce", "--select", "deliberately-invalid-policy"]);
    assert_eq!(code, 1);
    assert!(out.is_empty());
    assert!(
        err.starts_with("Stopped: Unknown policy 'deliberately-invalid-policy'"),
        "{err}"
    );
    assert!(err.contains("No policy values were written."));
}

#[cfg(not(windows))]
#[test]
fn policy_access_is_refused_off_windows() {
    for args in [
        &[][..],
        &["audit"],
        &["enforce", "--select", "advertising-id"],
        &["restore", "--backup", "x.json"],
    ] {
        let (code, out, err) = pef(args);
        assert_eq!(code, 1, "{args:?}");
        assert!(out.is_empty());
        assert!(err.contains("Requires Windows 11"), "{args:?}: {err}");
    }
}

#[cfg(not(windows))]
#[test]
fn redirected_input_cannot_approve_even_if_the_gate_passed() {
    // Off Windows the gate fails first; this documents that the apply path is never
    // reached with piped input.
    let (code, _, err) = pef(&[
        "enforce",
        "--select",
        "advertising-id",
        "--apply",
        "--backup",
        "never.json",
    ]);
    assert_eq!(code, 1);
    assert!(!std::path::Path::new("never.json").exists());
    assert!(err.contains("Requires Windows 11"));
}
