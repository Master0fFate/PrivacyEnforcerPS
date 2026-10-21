//! Pure policy logic for Privacy Enforcer.
//!
//! This crate contains the policy catalog, the backup format, the argument grammar and the
//! audit / plan / apply / restore engine. It performs no registry, process, network or
//! platform access: every side effect of enforcement is injected through the
//! [`PolicyStore`] trait, so the safety rules can be tested exhaustively with the
//! in-memory [`MemoryStore`]. The only file access lives in the explicit helpers of the
//! [`backup`] module.
//!
//! The rules the engine guarantees:
//!
//! * audit and plan never write;
//! * the complete plan is persisted before the first registry write;
//! * every write is read back and compared;
//! * a value changed by another program or administrator stops the operation instead of
//!   being overwritten;
//! * restore is idempotent and tolerates a partially applied plan.

#![forbid(unsafe_code)]

pub mod backup;
pub mod catalog;
pub mod engine;
pub mod error;
pub mod model;
pub mod options;
pub mod store;
pub mod time;

pub use backup::BackupError;
pub use catalog::{Category, ImpactLevel, Preset, PrivacyPolicy};
pub use engine::{ApplyOutcome, PrivacyEngine, RestoreOutcome, validate};
pub use error::Error;
pub use model::{
    AuditFinding, AuditState, AuditSummary, BACKUP_FORMAT_VERSION, PolicyChange, PolicyValue,
    PrivacyBackup,
};
pub use options::{Command, CommandOptions};
pub use store::{MemoryStore, PolicyStore, StoreError};

/// The crate version, shared by the command line and the desktop application.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
