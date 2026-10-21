//! The command line grammar, shared by the CLI and documented in the README.
//!
//! The grammar fails closed: unknown or repeated options, a selection on a read-only
//! command that would make no sense, or an enforce without an explicit selection are all
//! errors before any platform access happens.

use crate::catalog::{self, PrivacyPolicy};
use crate::error::Error;

/// The top-level command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    /// Read-only comparison of policies against their desired values (the default).
    Audit,
    /// Print the catalog or the presets.
    List,
    /// Preview, and with `--apply` enforce, an explicit selection.
    Enforce,
    /// Validate, preview, and with `--apply` restore a backup file.
    Restore,
    /// Print usage.
    Help,
    /// Print the version.
    Version,
}

impl Command {
    /// The word used on the command line.
    pub const fn as_str(self) -> &'static str {
        match self {
            Command::Audit => "audit",
            Command::List => "list",
            Command::Enforce => "enforce",
            Command::Restore => "restore",
            Command::Help => "help",
            Command::Version => "version",
        }
    }

    /// Commands that can never write.
    pub const fn is_read_only(self) -> bool {
        matches!(
            self,
            Command::Audit | Command::List | Command::Help | Command::Version
        )
    }

    /// The word a person must type to confirm a write.
    pub const fn confirmation_word(self) -> Option<&'static str> {
        match self {
            Command::Enforce => Some("ENFORCE"),
            Command::Restore => Some("RESTORE"),
            _ => None,
        }
    }

    fn parse(word: &str) -> Option<Command> {
        match word {
            "audit" => Some(Command::Audit),
            "list" => Some(Command::List),
            "enforce" => Some(Command::Enforce),
            "restore" => Some(Command::Restore),
            "help" | "--help" | "-h" => Some(Command::Help),
            "version" | "--version" | "-V" => Some(Command::Version),
            _ => None,
        }
    }
}

/// A parsed, validated command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOptions {
    /// The command.
    pub command: Command,
    /// Policy IDs from `--select`, trimmed, in the order given.
    pub ids: Vec<String>,
    /// Preset IDs from `--preset`, trimmed, in the order given.
    pub presets: Vec<String>,
    /// The `--backup` path.
    pub backup: Option<String>,
    /// Whether `--apply` was given. Without it enforce and restore only preview.
    pub apply: bool,
    /// Whether `--json` was given.
    pub json: bool,
    /// Whether `list --presets` was given.
    pub show_presets: bool,
}

const VALUE_OPTIONS: [&str; 3] = ["--select", "--backup", "--preset"];
const FLAG_OPTIONS: [&str; 3] = ["--apply", "--json", "--presets"];

impl CommandOptions {
    /// Parses the arguments after the program name.
    pub fn parse<I, S>(args: I) -> Result<CommandOptions, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_owned()).collect();
        let command = match args.first() {
            None => Command::Audit,
            Some(word) => Command::parse(word)
                .ok_or_else(|| Error::Usage(format!("Unknown command '{word}'. Run help.")))?,
        };

        let mut seen: Vec<&str> = Vec::new();
        let mut select = None;
        let mut preset = None;
        let mut backup = None;
        let mut apply = false;
        let mut json = false;
        let mut show_presets = false;

        let mut index = 1;
        while index < args.len() {
            let key = args[index].as_str();
            let known = VALUE_OPTIONS
                .iter()
                .chain(FLAG_OPTIONS.iter())
                .find(|option| **option == key)
                .copied();
            let Some(key) = known.filter(|k| !seen.contains(k)) else {
                return Err(Error::Usage(format!("Unknown or repeated option: {key}")));
            };
            seen.push(key);
            if VALUE_OPTIONS.contains(&key) {
                index += 1;
                let value = args
                    .get(index)
                    .filter(|v| !v.starts_with("--") && !v.trim().is_empty())
                    .ok_or_else(|| Error::Usage(format!("Missing value for {key}")))?;
                match key {
                    "--select" => select = Some(value.clone()),
                    "--preset" => preset = Some(value.clone()),
                    _ => backup = Some(value.clone()),
                }
            } else {
                match key {
                    "--apply" => apply = true,
                    "--json" => json = true,
                    _ => show_presets = true,
                }
            }
            index += 1;
        }

        if command.is_read_only() && (apply || backup.is_some()) {
            return Err(Error::Usage(
                "Read-only commands do not accept --apply or --backup.".into(),
            ));
        }
        if command == Command::Restore && (select.is_some() || preset.is_some()) {
            return Err(Error::Usage(
                "Restore uses the backup's policy selection.".into(),
            ));
        }
        if !command.is_read_only() && json {
            return Err(Error::Usage(
                "--json is only supported for read-only output.".into(),
            ));
        }
        if show_presets && command != Command::List {
            return Err(Error::Usage(
                "--presets is only supported with list.".into(),
            ));
        }
        if show_presets && (select.is_some() || preset.is_some()) {
            return Err(Error::Usage(
                "list --presets prints every preset and takes no selection.".into(),
            ));
        }

        let split = |value: Option<String>| -> Vec<String> {
            value
                .map(|v| v.split(',').map(|part| part.trim().to_owned()).collect())
                .unwrap_or_default()
        };
        let options = CommandOptions {
            command,
            ids: split(select),
            presets: split(preset),
            backup,
            apply,
            json,
            show_presets,
        };

        for preset in &options.presets {
            catalog::preset(preset)?;
        }
        if command == Command::Enforce && !options.has_selection() {
            return Err(Error::EmptySelection);
        }
        if options.has_selection() {
            options.policies()?;
        }
        if (command == Command::Restore || (command == Command::Enforce && apply))
            && options.backup.is_none()
        {
            return Err(Error::Usage(
                "This operation requires --backup PATH.".into(),
            ));
        }
        Ok(options)
    }

    /// True when `--select` or `--preset` narrowed the catalog.
    pub fn has_selection(&self) -> bool {
        !self.ids.is_empty() || !self.presets.is_empty()
    }

    /// The policies the command operates on, in catalog order.
    ///
    /// Without a selection this is the whole catalog, which is only ever used by the
    /// read-only commands: `parse` requires an explicit selection for enforce.
    pub fn policies(&self) -> Result<Vec<&'static PrivacyPolicy>, Error> {
        if !self.has_selection() {
            return Ok(catalog::all().iter().collect());
        }
        let mut ids: Vec<&str> = self.ids.iter().map(String::as_str).collect();
        for preset in &self.presets {
            ids.extend(
                catalog::expand_preset(catalog::preset(preset)?)
                    .iter()
                    .map(|p| p.id),
            );
        }
        let mut selected = catalog::select(ids)?;
        selected.sort_by_key(|policy| catalog_index(policy));
        Ok(selected)
    }
}

fn catalog_index(policy: &PrivacyPolicy) -> usize {
    catalog::all()
        .iter()
        .position(|candidate| candidate.id == policy.id)
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Result<CommandOptions, Error> {
        CommandOptions::parse(line.split(' ').filter(|s| !s.is_empty()))
    }

    #[test]
    fn no_arguments_means_audit() {
        let options = CommandOptions::parse(Vec::<String>::new()).unwrap();
        assert_eq!(options.command, Command::Audit);
        assert!(!options.apply && !options.json && !options.has_selection());
        assert_eq!(options.policies().unwrap().len(), catalog::all().len());
    }

    #[test]
    fn enforce_defaults_to_dry_run() {
        let options = parse("enforce --select advertising-id").unwrap();
        assert!(!options.apply);
        assert_eq!(options.ids, ["advertising-id"]);
        assert!(options.backup.is_none());
    }

    #[test]
    fn invalid_commands_fail_closed() {
        for line in [
            "enforce",
            "enforce --select unknown",
            "enforce --select advertising-id --apply",
            "enforce --preset nope",
            "enforce --preset baseline --apply",
            "audit --apply",
            "audit --backup x.json",
            "list --backup x.json",
            "help --apply",
            "version --backup x",
            "restore",
            "restore --backup a --select advertising-id",
            "restore --backup a --preset baseline",
            "restore --backup a --json",
            "enforce --select advertising-id --json",
            "audit --surprise",
            "audit --select",
            "audit --select --json",
            "audit --select ,",
            "list --json --json",
            "list --select advertising-id --select app-camera",
            "audit --presets",
            "list --presets --select advertising-id",
            "list --presets --preset baseline",
            "enforce --select advertising-id --backup",
            "enforce --select advertising-id --backup --apply",
            "frobnicate",
            "AUDIT",
            "--select advertising-id",
        ] {
            let err = parse(line).unwrap_err();
            assert!(
                matches!(
                    err,
                    Error::Usage(_)
                        | Error::UnknownPolicy(_)
                        | Error::UnknownPreset(_)
                        | Error::EmptySelection
                ),
                "{line}: {err}"
            );
        }
    }

    #[test]
    fn error_messages_are_specific() {
        assert_eq!(
            parse("audit --surprise").unwrap_err().to_string(),
            "Unknown or repeated option: --surprise"
        );
        assert_eq!(
            parse("list --json --json").unwrap_err().to_string(),
            "Unknown or repeated option: --json"
        );
        assert_eq!(
            parse("audit --select").unwrap_err().to_string(),
            "Missing value for --select"
        );
        assert_eq!(
            parse("audit --apply").unwrap_err().to_string(),
            "Read-only commands do not accept --apply or --backup."
        );
        assert_eq!(
            parse("restore --backup a --select advertising-id")
                .unwrap_err()
                .to_string(),
            "Restore uses the backup's policy selection."
        );
        assert_eq!(
            parse("enforce --select advertising-id --json")
                .unwrap_err()
                .to_string(),
            "--json is only supported for read-only output."
        );
        assert_eq!(
            parse("enforce --select advertising-id --apply")
                .unwrap_err()
                .to_string(),
            "This operation requires --backup PATH."
        );
        assert_eq!(
            parse("enforce").unwrap_err().to_string(),
            "Select at least one explicit policy ID."
        );
        assert_eq!(
            parse("frobnicate").unwrap_err().to_string(),
            "Unknown command 'frobnicate'. Run help."
        );
        assert!(
            parse("enforce --preset nope")
                .unwrap_err()
                .to_string()
                .starts_with("Unknown preset 'nope'")
        );
    }

    #[test]
    fn valid_command_lines_parse() {
        let options =
            parse("enforce --select advertising-id,clipboard-sync --apply --backup C:\\b\\x.json")
                .unwrap();
        assert_eq!(options.command, Command::Enforce);
        assert!(options.apply);
        assert_eq!(options.backup.as_deref(), Some("C:\\b\\x.json"));
        assert_eq!(options.ids, ["advertising-id", "clipboard-sync"]);

        let options = parse("restore --backup x.json").unwrap();
        assert_eq!(options.command, Command::Restore);
        assert!(!options.apply);
        assert_eq!(options.policies().unwrap().len(), catalog::all().len());

        let options = parse("restore --backup x.json --apply").unwrap();
        assert!(options.apply);

        let options = parse("list --json").unwrap();
        assert!(options.json);
        let options = parse("list --presets").unwrap();
        assert!(options.show_presets);
        let options = parse("audit --select app-camera --json").unwrap();
        assert_eq!(options.policies().unwrap()[0].id, "app-camera");

        for word in ["help", "--help", "-h"] {
            assert_eq!(parse(word).unwrap().command, Command::Help);
        }
        for word in ["version", "--version", "-V"] {
            assert_eq!(parse(word).unwrap().command, Command::Version);
        }
    }

    #[test]
    fn selection_values_are_trimmed_and_deduplicated_into_catalog_order() {
        let options = CommandOptions::parse([
            "enforce",
            "--select",
            " app-camera , advertising-id,app-camera ",
        ])
        .unwrap();
        assert_eq!(options.ids, ["app-camera", "advertising-id", "app-camera"]);
        let ids: Vec<&str> = options.policies().unwrap().iter().map(|p| p.id).collect();
        assert_eq!(
            ids,
            ["advertising-id", "app-camera"],
            "catalog order, deduplicated"
        );
    }

    #[test]
    fn presets_and_selection_combine() {
        let options = parse("enforce --preset edge --select advertising-id").unwrap();
        let ids: Vec<&str> = options.policies().unwrap().iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), 5);
        assert_eq!(ids[0], "advertising-id");
        assert!(ids[1..].iter().all(|id| id.starts_with("edge-")));

        let options = parse("enforce --preset baseline,edge").unwrap();
        let expected = catalog::expand_preset(catalog::preset("baseline").unwrap()).len()
            + catalog::expand_preset(catalog::preset("edge").unwrap()).len();
        assert_eq!(options.policies().unwrap().len(), expected);

        let options = parse("audit --preset strict").unwrap();
        assert_eq!(options.policies().unwrap().len(), catalog::all().len());
    }

    #[test]
    fn option_values_may_not_look_like_options() {
        let err = parse("enforce --select --apply").unwrap_err();
        assert_eq!(err.to_string(), "Missing value for --select");
        let options = CommandOptions::parse([
            "enforce",
            "--select",
            "advertising-id",
            "--backup",
            "-dash.json",
            "--apply",
        ])
        .unwrap();
        assert_eq!(options.backup.as_deref(), Some("-dash.json"));
    }

    #[test]
    fn command_metadata() {
        assert_eq!(Command::Enforce.confirmation_word(), Some("ENFORCE"));
        assert_eq!(Command::Restore.confirmation_word(), Some("RESTORE"));
        assert_eq!(Command::Audit.confirmation_word(), None);
        assert!(Command::Audit.is_read_only() && Command::Version.is_read_only());
        assert!(!Command::Enforce.is_read_only() && !Command::Restore.is_read_only());
        assert_eq!(Command::List.as_str(), "list");
    }
}
