//! The `pef` command line, written as a library so the whole workflow can be tested
//! in-process with an in-memory store and scripted confirmations.
//!
//! Exit codes: `0` for a successful read, preview, write or a cancellation; `1` for invalid
//! input, an unsupported platform or a stopped operation; `2` when an audit contains
//! unknown values.

#![forbid(unsafe_code)]

use std::fmt::Write as _;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use pef_core::{
    AuditState, AuditSummary, Command, CommandOptions, Error, PolicyStore, PrivacyBackup,
    PrivacyEngine, backup, catalog,
};

/// Successful read, preview, write or cancellation.
pub const EXIT_OK: u8 = 0;
/// Invalid input, unsupported platform or a stopped operation.
pub const EXIT_STOPPED: u8 = 1;
/// The audit contains at least one unknown value.
pub const EXIT_UNKNOWN_FINDINGS: u8 = 2;

/// The parts of the environment the workflow touches, so tests can substitute them.
pub trait Host {
    /// Checks the operating-system gate and opens the policy store. The string
    /// describes the system for the audit header.
    fn open_store(&mut self) -> Result<(String, Box<dyn PolicyStore>), String>;
    /// The computer name recorded in and checked against backups.
    fn machine_name(&self) -> String;
    /// Whether the process may write machine-wide policies.
    fn is_elevated(&self) -> bool;
    /// Whether standard input is an interactive terminal.
    fn input_is_terminal(&self) -> bool;
    /// Reads one line of confirmation; `None` at end of input.
    fn read_line(&mut self) -> io::Result<Option<String>>;
}

/// The real environment.
#[derive(Debug, Default)]
pub struct SystemHost;

impl Host for SystemHost {
    fn open_store(&mut self) -> Result<(String, Box<dyn PolicyStore>), String> {
        let version = pef_platform::require_supported_windows().map_err(|e| e.to_string())?;
        let store = pef_platform::open_policy_store().map_err(|e| e.to_string())?;
        Ok((version.describe(), store))
    }

    fn machine_name(&self) -> String {
        pef_platform::machine_name()
    }

    fn is_elevated(&self) -> bool {
        pef_platform::is_elevated()
    }

    fn input_is_terminal(&self) -> bool {
        io::IsTerminal::is_terminal(&io::stdin())
    }

    fn read_line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        let read = io::stdin().read_line(&mut line)?;
        Ok((read > 0).then(|| line.trim_end_matches(['\r', '\n']).to_owned()))
    }
}

/// Why a run stopped, and whether registry writes may already have happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The message, without the `Stopped:` prefix.
    pub message: String,
    /// True when the failure happened during apply or restore.
    pub may_have_written: bool,
}

impl Failure {
    fn before_write(message: impl ToString) -> Self {
        Failure {
            message: message.to_string(),
            may_have_written: false,
        }
    }

    fn during_write(error: Error) -> Self {
        Failure {
            may_have_written: !error.is_pre_write(),
            message: error.to_string(),
        }
    }

    /// The text printed on standard error.
    pub fn render(&self) -> String {
        if self.may_have_written {
            format!(
                "Stopped: {}\nThe operation began before it stopped. Retain the backup and audit before retrying; earlier writes may have completed.",
                self.message
            )
        } else {
            format!("Stopped: {}\nNo policy values were written.", self.message)
        }
    }
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Failure::before_write(error)
    }
}

impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        Failure::before_write(format!("output failed: {error}"))
    }
}

/// Usage text.
pub const HELP: &str = "pef: audit-first Windows privacy policies

  list [--select ID,ID] [--preset NAME,NAME] [--json]
  list --presets [--json]
  audit [--select ID,ID] [--preset NAME,NAME] [--json]        (default)
  enforce (--select ID,ID | --preset NAME,NAME) [--apply --backup NEW_FILE.json]
  restore --backup FILE.json [--apply]
  help | version

Without --apply, enforce and restore only preview. Apply requires an administrator
terminal and a typed confirmation. Use a private local backup path. Review each
policy's impact with list before selecting it; nothing is selected implicitly and a
preset is only a shorthand for the IDs the preview shows.";

/// Runs one command line. Writes normal output to `out`; errors are returned.
pub fn run(args: &[String], host: &mut dyn Host, out: &mut dyn Write) -> Result<u8, Failure> {
    let options = CommandOptions::parse(args)?;
    match options.command {
        Command::Help => {
            writeln!(out, "{HELP}")?;
            Ok(EXIT_OK)
        }
        Command::Version => {
            writeln!(out, "pef {}", pef_core::VERSION)?;
            Ok(EXIT_OK)
        }
        Command::List => {
            list(&options, out)?;
            Ok(EXIT_OK)
        }
        Command::Audit => audit(&options, host, out),
        Command::Enforce | Command::Restore => write_workflow(&options, host, out),
    }
}

fn list(options: &CommandOptions, out: &mut dyn Write) -> Result<(), Failure> {
    if options.show_presets {
        if options.json {
            let presets: Vec<serde_json::Value> = catalog::presets()
                .iter()
                .map(|preset| {
                    let ids: Vec<&str> = catalog::expand_preset(preset)
                        .iter()
                        .map(|p| p.id)
                        .collect();
                    serde_json::json!({
                        "Id": preset.id,
                        "Title": preset.title,
                        "Description": preset.description,
                        "Policies": ids,
                    })
                })
                .collect();
            writeln!(
                out,
                "{}",
                serde_json::to_string_pretty(&presets).unwrap_or_default()
            )?;
        } else {
            for preset in catalog::presets() {
                let policies = catalog::expand_preset(preset);
                writeln!(
                    out,
                    "{}: {} ({} policies)",
                    preset.id,
                    preset.title,
                    policies.len()
                )?;
                writeln!(out, "  {}", preset.description)?;
                let ids: Vec<&str> = policies.iter().map(|p| p.id).collect();
                writeln!(out, "  Policies: {}", ids.join(", "))?;
            }
        }
        return Ok(());
    }
    let policies = options.policies()?;
    if options.json {
        writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(&policies).unwrap_or_default()
        )?;
        return Ok(());
    }
    let mut current_category = None;
    for policy in policies {
        if current_category != Some(policy.category) {
            current_category = Some(policy.category);
            writeln!(
                out,
                "== {}: {}",
                policy.category.label(),
                policy.category.description()
            )?;
        }
        writeln!(
            out,
            "{}: {} = {}",
            policy.id,
            policy.registry_path(),
            policy.desired
        )?;
        writeln!(out, "  {} ({})", policy.title, policy.impact_level.label())?;
        writeln!(out, "  Impact: {}", policy.impact)?;
        writeln!(out, "  Source: {}", policy.source)?;
    }
    Ok(())
}

fn audit(
    options: &CommandOptions,
    host: &mut dyn Host,
    out: &mut dyn Write,
) -> Result<u8, Failure> {
    let policies = options.policies()?;
    let (system, store) = host.open_store().map_err(Failure::before_write)?;
    let engine = PrivacyEngine::new(store);
    let findings = engine.audit(policies);
    let summary = AuditSummary::of(&findings);
    if options.json {
        let report = serde_json::json!({
            "System": system,
            "Summary": summary,
            "Findings": findings,
        });
        writeln!(
            out,
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        )?;
    } else {
        writeln!(
            out,
            "Read-only policy registry audit: {system}. Configured is not proof of effective enforcement. MDM/GPO, per-app overrides and desktop apps need separate review."
        )?;
        for finding in &findings {
            let current = match finding.current {
                Some(value) if value.exists => value.to_string(),
                _ => "not configured/unknown".to_owned(),
            };
            writeln!(
                out,
                "{}: {}; current={current}, desired={}",
                finding.id, finding.state, finding.desired
            )?;
            writeln!(out, "  {}", finding.impact)?;
            if let Some(error) = &finding.error {
                writeln!(out, "  Error: {error}")?;
            }
        }
        writeln!(
            out,
            "Summary: {} configured, {} different or not configured, {} unknown ({} policies).",
            summary.configured,
            summary.different,
            summary.unknown,
            summary.total()
        )?;
    }
    Ok(if summary.unknown > 0 {
        EXIT_UNKNOWN_FINDINGS
    } else {
        EXIT_OK
    })
}

fn describe_plan(
    options: &CommandOptions,
    backup: &PrivacyBackup,
    out: &mut dyn Write,
) -> Result<(), Failure> {
    let mode = if options.apply {
        "Requested operation"
    } else {
        "DRY RUN"
    };
    writeln!(
        out,
        "{mode}: {}; {} selected changes",
        options.command.as_str(),
        backup.changes.len()
    )?;
    for change in &backup.changes {
        let policy = catalog::get(&change.id)?;
        writeln!(
            out,
            "{}: before={}, enforced={}",
            change.id, change.before, change.desired
        )?;
        writeln!(out, "  {} ({})", policy.title, policy.impact_level.label())?;
        writeln!(out, "  {}", policy.impact)?;
    }
    if backup.changes.is_empty() {
        let note = match options.command {
            Command::Restore => "The backup records no changes; there is nothing to restore.",
            _ => "Nothing to change: every selected policy already holds its desired value.",
        };
        writeln!(out, "{note}")?;
    }
    Ok(())
}

fn write_workflow(
    options: &CommandOptions,
    host: &mut dyn Host,
    out: &mut dyn Write,
) -> Result<u8, Failure> {
    let (_, store) = host.open_store().map_err(Failure::before_write)?;
    let mut engine = PrivacyEngine::new(store);
    let machine = host.machine_name();
    let backup_path = options.backup.as_deref().map(Path::new);
    let plan = match options.command {
        Command::Restore => {
            let path = backup_path.expect("parser requires --backup for restore");
            backup::load(path).map_err(Error::from)?
        }
        _ => engine.plan(options.policies()?, &machine)?,
    };
    pef_core::validate(&plan, &machine)?;
    describe_plan(options, &plan, out)?;
    if options.command == Command::Restore {
        engine.restore(&plan, &machine, true)?;
    }
    if !options.apply || plan.changes.is_empty() {
        return Ok(EXIT_OK);
    }
    if !host.is_elevated() {
        return Err(Failure::before_write(
            "Apply/restore requires an elevated terminal. Audit and dry-run do not.",
        ));
    }
    if !host.input_is_terminal() {
        return Err(Failure::before_write(
            "Interactive confirmation required. Re-run in a terminal; redirected input cannot approve changes.",
        ));
    }
    let word = options
        .command
        .confirmation_word()
        .expect("enforce and restore have confirmation words");
    writeln!(
        out,
        "These are machine-wide policies. On organization-managed devices, consult your administrator first. No guarantee of anonymity or zero telemetry. Restart affected apps; some policies may need sign-out/restart."
    )?;
    write!(out, "Type {word} to confirm only these policies: ")?;
    out.flush()?;
    let answer = host.read_line().map_err(Failure::from)?;
    if answer.as_deref() != Some(word) {
        writeln!(out, "Cancelled. No policy writes.")?;
        return Ok(EXIT_OK);
    }
    match options.command {
        Command::Restore => {
            let outcome = engine
                .restore(&plan, &machine, false)
                .map_err(Failure::during_write)?;
            writeln!(
                out,
                "Registry values verified: {} restored, {} already held their original value. Audit effective behavior separately; no services, security controls, tasks or network settings were changed.",
                outcome.restored, outcome.already_restored
            )?;
        }
        _ => {
            let path = absolute(backup_path.expect("parser requires --backup for apply"));
            writeln!(
                out,
                "Recovery file: {}. Keep it private and retain it if anything fails.",
                path.display()
            )?;
            let outcome = engine
                .apply(&plan, &machine, |plan| backup::save(&path, plan))
                .map_err(Failure::during_write)?;
            writeln!(
                out,
                "Registry values verified: {} written. Audit effective behavior separately; no services, security controls, tasks or network settings were changed.",
                outcome.written
            )?;
        }
    }
    Ok(EXIT_OK)
}

fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Formats one finding line for user interfaces that reuse the CLI wording.
pub fn finding_line(finding: &pef_core::AuditFinding) -> String {
    let mut line = String::new();
    let _ = write!(line, "{}: {}", finding.id, finding.state);
    if finding.state == AuditState::Unknown {
        let _ = write!(
            line,
            " ({})",
            finding.error.as_deref().unwrap_or("unreadable")
        );
    }
    line
}
