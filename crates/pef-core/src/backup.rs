//! Reading and writing recovery files.
//!
//! A backup is pretty-printed JSON, at most 64 KiB, created exclusively (never
//! overwriting an earlier recovery point), flushed to disk and read back before the
//! engine is allowed to write a single registry value.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::model::PrivacyBackup;
use crate::time;

/// Upper bound on the size of a backup file. A real backup is a few kilobytes.
pub const MAX_BACKUP_BYTES: u64 = 64 * 1024;

/// Failures while saving or loading a recovery file.
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    /// The file could not be created. It must be a new file inside an existing folder.
    #[error(
        "Backup file '{path}' could not be created (it must be a new file in an existing private folder): {source}"
    )]
    Create {
        /// The requested path.
        path: PathBuf,
        /// The platform error.
        #[source]
        source: io::Error,
    },
    /// The file was created but could not be written, flushed or verified.
    #[error("Backup file '{path}' could not be written: {source}")]
    Write {
        /// The requested path.
        path: PathBuf,
        /// The platform error.
        #[source]
        source: io::Error,
    },
    /// The file could not be opened for reading.
    #[error("Backup file '{path}' could not be opened: {source}")]
    Open {
        /// The requested path.
        path: PathBuf,
        /// The platform error.
        #[source]
        source: io::Error,
    },
    /// The file is larger than [`MAX_BACKUP_BYTES`].
    #[error("Backup file '{path}' exceeds the {limit} byte limit.")]
    TooLarge {
        /// The requested path.
        path: PathBuf,
        /// The limit that was exceeded.
        limit: u64,
    },
    /// The content is not a backup this tool wrote.
    #[error("Backup file '{path}' is not a valid Privacy Enforcer backup: {reason}")]
    Malformed {
        /// The requested path.
        path: PathBuf,
        /// What was wrong.
        reason: String,
    },
    /// Any other failure, used by callers that persist backups elsewhere.
    #[error("{0}")]
    Other(String),
}

/// Serializes a backup as the pretty-printed JSON document stored on disk.
pub fn to_json(backup: &PrivacyBackup) -> String {
    serde_json::to_string_pretty(backup).expect("backup types always serialize")
}

/// Parses and structurally validates backup JSON.
///
/// Unknown or duplicate fields, missing fields, wrong types and malformed timestamps are
/// rejected. Policy IDs, the version and the machine are checked separately by
/// [`crate::engine::validate`].
pub fn parse(text: &str) -> Result<PrivacyBackup, String> {
    let backup: PrivacyBackup = serde_json::from_str(text).map_err(|err| err.to_string())?;
    if !time::is_rfc3339_timestamp(&backup.created) {
        return Err(format!(
            "'Created' is not an RFC 3339 timestamp: {:?}",
            backup.created
        ));
    }
    Ok(backup)
}

/// Writes a new backup file and verifies it by reading it back.
///
/// The file is created exclusively so an earlier recovery point is never replaced. The
/// parent folder must already exist; the tool never creates folders or changes permissions.
pub fn save(path: &Path, backup: &PrivacyBackup) -> Result<(), BackupError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| BackupError::Create {
            path: path.to_path_buf(),
            source,
        })?;
    let written = write_and_verify(&mut file, path, backup);
    drop(file);
    if let Err(err) = written {
        // The file is ours: it was created exclusively a moment ago and nothing was
        // applied yet, so a partial recovery file has no value and would block a retry.
        let _ = fs::remove_file(path);
        return Err(err);
    }
    Ok(())
}

fn write_and_verify(
    file: &mut File,
    path: &Path,
    backup: &PrivacyBackup,
) -> Result<(), BackupError> {
    let json = to_json(backup);
    let write_error = |source: io::Error| BackupError::Write {
        path: path.to_path_buf(),
        source,
    };
    file.write_all(json.as_bytes()).map_err(write_error)?;
    file.sync_all().map_err(write_error)?;
    let reloaded = load(path).map_err(|err| {
        write_error(io::Error::other(format!(
            "verification failed after writing: {err}"
        )))
    })?;
    if &reloaded != backup {
        return Err(write_error(io::Error::other(
            "verification failed: the file does not match the plan",
        )));
    }
    Ok(())
}

/// Reads and validates a backup file.
pub fn load(path: &Path) -> Result<PrivacyBackup, BackupError> {
    let open_error = |source: io::Error| BackupError::Open {
        path: path.to_path_buf(),
        source,
    };
    let file = File::open(path).map_err(open_error)?;
    let too_large = || BackupError::TooLarge {
        path: path.to_path_buf(),
        limit: MAX_BACKUP_BYTES,
    };
    if file.metadata().map_err(open_error)?.len() > MAX_BACKUP_BYTES {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    file.take(MAX_BACKUP_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(open_error)?;
    if bytes.len() as u64 > MAX_BACKUP_BYTES {
        return Err(too_large());
    }
    let malformed = |reason: String| BackupError::Malformed {
        path: path.to_path_buf(),
        reason,
    };
    let text = std::str::from_utf8(&bytes).map_err(|err| malformed(format!("not UTF-8: {err}")))?;
    parse(text).map_err(malformed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PolicyChange, PolicyValue};

    fn sample() -> PrivacyBackup {
        PrivacyBackup {
            version: 1,
            machine: "test-machine".into(),
            created: "2026-09-30T00:00:00Z".into(),
            changes: vec![PolicyChange {
                id: "clipboard-sync".into(),
                before: PolicyValue::ABSENT,
                desired: 0,
            }],
        }
    }

    #[test]
    fn parse_round_trips_and_checks_created() {
        let json = to_json(&sample());
        assert!(
            json.contains("\n  \"Version\": 1"),
            "pretty printed: {json}"
        );
        assert_eq!(parse(&json).unwrap(), sample());
        let bad = json.replace("2026-09-30T00:00:00Z", "yesterday");
        let err = parse(&bad).unwrap_err();
        assert!(err.contains("RFC 3339"), "{err}");
    }

    #[test]
    fn parse_rejects_dotnet_style_duplicate_and_incomplete_values() {
        for before in [
            "{}",
            r#"{"Exists":true}"#,
            r#"{"Value":0}"#,
            r#"{"Exists":true,"Value":1,"Value":0}"#,
        ] {
            let text = format!(
                r#"{{"Version":1,"Machine":"test-machine","Created":"2026-09-30T00:00:00Z","Changes":[{{"Id":"clipboard-sync","Desired":0,"Before":{before}}}]}}"#
            );
            assert!(parse(&text).is_err(), "{before} must be rejected");
        }
        assert!(parse(r#"{"unknown":42}"#).is_err());
        assert!(parse("null").is_err());
        assert!(parse("").is_err());
        assert!(parse(&format!("{} trailing", to_json(&sample()))).is_err());
    }

    #[test]
    fn parse_accepts_files_written_by_the_dotnet_release() {
        let dotnet = r#"{
  "Version": 1,
  "Machine": "test-machine",
  "Created": "2026-09-30T12:34:56.1234567+00:00",
  "Changes": [
    {
      "Id": "advertising-id",
      "Before": {
        "Exists": true,
        "Value": 0
      },
      "Desired": 1
    }
  ]
}"#;
        let backup = parse(dotnet).unwrap();
        assert_eq!(backup.changes[0].before, PolicyValue::present(0));
        assert_eq!(backup.created, "2026-09-30T12:34:56.1234567+00:00");
    }

    #[test]
    fn save_creates_verifies_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("backup.json");
        save(&path, &sample()).unwrap();
        assert_eq!(load(&path).unwrap(), sample());
        let err = save(&path, &sample()).unwrap_err();
        assert!(matches!(err, BackupError::Create { .. }), "{err}");
        assert!(err.to_string().contains("new file"));
        assert_eq!(load(&path).unwrap(), sample(), "existing file untouched");
    }

    #[test]
    fn save_requires_an_existing_parent_folder() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing").join("backup.json");
        let err = save(&path, &sample()).unwrap_err();
        assert!(matches!(err, BackupError::Create { .. }));
        assert!(!path.exists());
        assert!(
            !dir.path().join("missing").exists(),
            "folders are never created"
        );
    }

    #[test]
    fn load_rejects_oversized_and_malformed_files() {
        let dir = tempfile::tempdir().unwrap();
        let big = dir.path().join("big.json");
        fs::write(&big, " ".repeat(65_537)).unwrap();
        let err = load(&big).unwrap_err();
        assert!(
            matches!(err, BackupError::TooLarge { limit: 65_536, .. }),
            "{err}"
        );

        let bad = dir.path().join("bad.json");
        fs::write(&bad, r#"{"unknown":42}"#).unwrap();
        let err = load(&bad).unwrap_err();
        assert!(matches!(err, BackupError::Malformed { .. }), "{err}");

        let binary = dir.path().join("binary.json");
        fs::write(&binary, [0xff, 0xfe, 0x00]).unwrap();
        assert!(matches!(
            load(&binary).unwrap_err(),
            BackupError::Malformed { .. }
        ));

        let missing = dir.path().join("missing.json");
        assert!(matches!(
            load(&missing).unwrap_err(),
            BackupError::Open { .. }
        ));
        let err = load(dir.path()).unwrap_err();
        assert!(
            matches!(
                err,
                BackupError::Open { .. } | BackupError::Malformed { .. }
            ),
            "{err}"
        );
    }

    #[test]
    fn exactly_the_limit_is_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("limit.json");
        let mut json = to_json(&sample());
        let padding = usize::try_from(MAX_BACKUP_BYTES).unwrap() - json.len();
        json.push_str(&" ".repeat(padding));
        assert_eq!(json.len() as u64, MAX_BACKUP_BYTES);
        fs::write(&path, &json).unwrap();
        assert_eq!(load(&path).unwrap(), sample());
    }

    #[test]
    fn error_messages_name_the_path() {
        let err = BackupError::TooLarge {
            path: PathBuf::from("x.json"),
            limit: 10,
        };
        assert_eq!(
            err.to_string(),
            "Backup file 'x.json' exceeds the 10 byte limit."
        );
        assert_eq!(
            BackupError::Other("disk full".into()).to_string(),
            "disk full"
        );
    }
}
