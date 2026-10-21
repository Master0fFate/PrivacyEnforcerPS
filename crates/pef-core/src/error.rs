//! Error type shared by the catalog, argument grammar and engine.

use crate::backup::BackupError;
use crate::store::StoreError;

/// Everything that can stop an operation. Each message is written for the person running
/// the tool and names what to do next.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A policy ID that is not in the catalog.
    #[error("Unknown policy '{0}'. Run list for valid IDs.")]
    UnknownPolicy(String),
    /// A preset name that is not in the catalog.
    #[error("Unknown preset '{0}'. Run list --presets for valid names.")]
    UnknownPreset(String),
    /// Enforcement needs an explicit selection; nothing is ever selected implicitly.
    #[error("Select at least one explicit policy ID.")]
    EmptySelection,
    /// Invalid command line.
    #[error("{0}")]
    Usage(String),
    /// A backup failed structural validation against this machine and the catalog.
    #[error("{0}")]
    InvalidBackup(String),
    /// A value differs from the plan before anything was written.
    #[error(
        "Conflict for {id}: current value changed outside this plan. No automatic overwrite; audit first."
    )]
    Conflict {
        /// Policy ID.
        id: String,
    },
    /// A value changed between the pre-flight check and its write.
    #[error(
        "{id} changed during apply. Stop and audit; use the saved backup to recover earlier writes."
    )]
    ChangedDuringApply {
        /// Policy ID.
        id: String,
    },
    /// A value changed between the pre-flight check and its restore.
    #[error("{id} changed during restore. Stop and audit.")]
    ChangedDuringRestore {
        /// Policy ID.
        id: String,
    },
    /// A write was acknowledged but reading it back returned something else.
    #[error("Read-back failed for {id}. Use the saved backup to recover.")]
    ReadBack {
        /// Policy ID.
        id: String,
    },
    /// A restore write was acknowledged but reading it back returned something else.
    #[error("Read-back failed restoring {id}; retain the backup.")]
    RestoreReadBack {
        /// Policy ID.
        id: String,
    },
    /// The store failed.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// The backup file could not be written or read.
    #[error(transparent)]
    Backup(#[from] BackupError),
}

impl Error {
    /// True when the failure happened before any registry write could have occurred.
    ///
    /// Backup errors count as pre-write because the engine persists the recovery file
    /// before its first write. Store errors are conservatively reported as "may have
    /// written" because they can occur mid-operation.
    pub fn is_pre_write(&self) -> bool {
        matches!(
            self,
            Error::UnknownPolicy(_)
                | Error::UnknownPreset(_)
                | Error::EmptySelection
                | Error::Usage(_)
                | Error::InvalidBackup(_)
                | Error::Conflict { .. }
                | Error::Backup(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_user_facing() {
        assert_eq!(
            Error::Conflict {
                id: "app-camera".into()
            }
            .to_string(),
            "Conflict for app-camera: current value changed outside this plan. No automatic overwrite; audit first."
        );
        assert_eq!(
            Error::ReadBack { id: "x".into() }.to_string(),
            "Read-back failed for x. Use the saved backup to recover."
        );
        assert_eq!(Error::Usage("bad".into()).to_string(), "bad");
        assert_eq!(
            Error::EmptySelection.to_string(),
            "Select at least one explicit policy ID."
        );
    }

    #[test]
    fn pre_write_classification() {
        assert!(Error::Usage("x".into()).is_pre_write());
        assert!(Error::Conflict { id: "x".into() }.is_pre_write());
        assert!(!Error::ChangedDuringApply { id: "x".into() }.is_pre_write());
        assert!(!Error::ReadBack { id: "x".into() }.is_pre_write());
        assert!(Error::Backup(BackupError::Other("x".into())).is_pre_write());
        assert!(!Error::Store(StoreError::OutsideCatalog { id: "x".into() }).is_pre_write());
    }
}
