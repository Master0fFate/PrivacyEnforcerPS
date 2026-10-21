//! Data types shared by the engine, the backup format and the user interfaces.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, de};

/// Version of the backup file format produced by this crate.
pub const BACKUP_FORMAT_VERSION: u32 = 1;

/// The state of one registry value: absent, or present with a `DWORD`.
///
/// Serialized with the field names of the original backup format so recovery files
/// created by earlier releases remain restorable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct PolicyValue {
    /// Whether the value exists at all.
    pub exists: bool,
    /// The `DWORD` when it exists; zero when absent.
    #[serde(deserialize_with = "deserialize_dword")]
    pub value: u32,
}

impl PolicyValue {
    /// A value that does not exist in the registry.
    pub const ABSENT: PolicyValue = PolicyValue {
        exists: false,
        value: 0,
    };

    /// A value that exists with the given `DWORD`.
    pub const fn present(value: u32) -> PolicyValue {
        PolicyValue {
            exists: true,
            value,
        }
    }
}

impl fmt::Display for PolicyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.exists {
            write!(f, "{}", self.value)
        } else {
            f.write_str("absent")
        }
    }
}

/// Accepts the unsigned `DWORD` range and, for files written by the earlier .NET release,
/// the two's complement form of values at or above `0x8000_0000`.
fn deserialize_dword<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = i64::deserialize(deserializer)?;
    if let Ok(value) = u32::try_from(raw) {
        Ok(value)
    } else if let Ok(signed) = i32::try_from(raw) {
        Ok(signed as u32)
    } else {
        Err(de::Error::custom(format!(
            "value {raw} is outside the DWORD range"
        )))
    }
}

/// One planned or recorded change: the policy, what the registry held before, and the
/// enforced value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct PolicyChange {
    /// Catalog policy ID. Backups never carry registry paths.
    pub id: String,
    /// The value before enforcement, restored on rollback.
    pub before: PolicyValue,
    /// The enforced value; must match the catalog.
    pub desired: u32,
}

/// A complete, self-describing recovery plan.
///
/// The same type is used for a plan preview, the file written before the first registry
/// write, and the input of a restore.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", deny_unknown_fields)]
pub struct PrivacyBackup {
    /// File format version; see [`BACKUP_FORMAT_VERSION`].
    pub version: u32,
    /// Name of the computer the plan was created on. A guard against mistakes, not identity.
    pub machine: String,
    /// RFC 3339 timestamp of plan creation.
    pub created: String,
    /// The changes in apply order. Restore walks them in reverse.
    pub changes: Vec<PolicyChange>,
}

impl PrivacyBackup {
    /// IDs of the policies the plan touches, in apply order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.changes.iter().map(|change| change.id.as_str())
    }
}

/// Result of comparing one policy against its desired value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum AuditState {
    /// The stored registry value equals the desired value.
    Configured,
    /// The value is absent or different. Not necessarily a problem: nothing was selected.
    DifferentOrNotConfigured,
    /// The value could not be read or has an unexpected type. Never treated as protected.
    Unknown,
}

impl AuditState {
    /// The stable string used in text and JSON output.
    pub const fn as_str(self) -> &'static str {
        match self {
            AuditState::Configured => "Configured",
            AuditState::DifferentOrNotConfigured => "DifferentOrNotConfigured",
            AuditState::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for AuditState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One line of an audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuditFinding {
    /// Catalog policy ID.
    pub id: &'static str,
    /// Comparison result.
    pub state: AuditState,
    /// The value that was read, or `None` when reading failed.
    pub current: Option<PolicyValue>,
    /// The value enforcement would write.
    pub desired: u32,
    /// Impact statement from the catalog.
    pub impact: &'static str,
    /// Read error, when the state is [`AuditState::Unknown`].
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_value_display_and_constructors() {
        assert_eq!(PolicyValue::ABSENT.to_string(), "absent");
        assert_eq!(PolicyValue::present(2).to_string(), "2");
        assert_eq!(
            PolicyValue::ABSENT,
            PolicyValue {
                exists: false,
                value: 0
            }
        );
        assert_ne!(
            PolicyValue::ABSENT,
            PolicyValue {
                exists: false,
                value: 1
            }
        );
    }

    #[test]
    fn policy_value_json_uses_pascal_case() {
        let json = serde_json::to_string(&PolicyValue::present(7)).unwrap();
        assert_eq!(json, r#"{"Exists":true,"Value":7}"#);
        let back: PolicyValue = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PolicyValue::present(7));
    }

    #[test]
    fn policy_value_requires_both_fields_and_rejects_duplicates_and_extras() {
        for bad in [
            "{}",
            r#"{"Exists":true}"#,
            r#"{"Value":0}"#,
            r#"{"Exists":true,"Value":1,"Value":0}"#,
            r#"{"Exists":true,"Exists":false,"Value":1}"#,
            r#"{"Exists":true,"Value":1,"Extra":1}"#,
            r#"{"exists":true,"value":1}"#,
            r#"{"Exists":1,"Value":1}"#,
            r#"{"Exists":true,"Value":"1"}"#,
            r#"{"Exists":true,"Value":1.0}"#,
            r#"{"Exists":true,"Value":4294967296}"#,
            r#"{"Exists":true,"Value":-2147483649}"#,
            "null",
            "[]",
        ] {
            assert!(
                serde_json::from_str::<PolicyValue>(bad).is_err(),
                "{bad} must be rejected"
            );
        }
    }

    #[test]
    fn dword_accepts_unsigned_range_and_dotnet_signed_form() {
        let max: PolicyValue =
            serde_json::from_str(r#"{"Exists":true,"Value":4294967295}"#).unwrap();
        assert_eq!(max.value, u32::MAX);
        let signed: PolicyValue = serde_json::from_str(r#"{"Exists":true,"Value":-1}"#).unwrap();
        assert_eq!(signed.value, u32::MAX);
        let min: PolicyValue =
            serde_json::from_str(r#"{"Exists":true,"Value":-2147483648}"#).unwrap();
        assert_eq!(min.value, 0x8000_0000);
    }

    #[test]
    fn audit_state_strings_are_stable() {
        assert_eq!(AuditState::Configured.as_str(), "Configured");
        assert_eq!(
            AuditState::DifferentOrNotConfigured.to_string(),
            "DifferentOrNotConfigured"
        );
        assert_eq!(
            serde_json::to_string(&AuditState::Unknown).unwrap(),
            "\"Unknown\""
        );
    }

    #[test]
    fn backup_json_round_trips_with_documented_field_names() {
        let backup = PrivacyBackup {
            version: 1,
            machine: "pc".into(),
            created: "2026-01-02T03:04:05Z".into(),
            changes: vec![PolicyChange {
                id: "advertising-id".into(),
                before: PolicyValue::ABSENT,
                desired: 1,
            }],
        };
        let json = serde_json::to_value(&backup).unwrap();
        assert_eq!(json["Version"], 1);
        assert_eq!(json["Machine"], "pc");
        assert_eq!(json["Created"], "2026-01-02T03:04:05Z");
        assert_eq!(json["Changes"][0]["Id"], "advertising-id");
        assert_eq!(json["Changes"][0]["Before"]["Exists"], false);
        assert_eq!(json["Changes"][0]["Desired"], 1);
        let back: PrivacyBackup = serde_json::from_value(json).unwrap();
        assert_eq!(back, backup);
        assert_eq!(back.ids().collect::<Vec<_>>(), ["advertising-id"]);
    }

    #[test]
    fn backup_rejects_unknown_fields_null_changes_and_missing_fields() {
        for bad in [
            r#"{"Version":1,"Machine":"pc","Created":"2026-01-02T03:04:05Z","Changes":[],"Extra":1}"#,
            r#"{"Version":1,"Machine":"pc","Created":"2026-01-02T03:04:05Z","Changes":null}"#,
            r#"{"Version":1,"Machine":"pc","Created":"2026-01-02T03:04:05Z"}"#,
            r#"{"Version":"1","Machine":"pc","Created":"2026-01-02T03:04:05Z","Changes":[]}"#,
            r#"{"Version":-1,"Machine":"pc","Created":"2026-01-02T03:04:05Z","Changes":[]}"#,
            r#"{"Version":1,"Machine":null,"Created":"2026-01-02T03:04:05Z","Changes":[]}"#,
            r#"{"Version":1,"Machine":"pc","Created":"2026-01-02T03:04:05Z","Changes":[{"Id":"x","Before":{"Exists":false,"Value":0}}]}"#,
            r#"{"Version":1,"Machine":"pc","Created":"2026-01-02T03:04:05Z","Changes":[{"Id":"x","Before":{"Exists":false,"Value":0},"Desired":-1}]}"#,
            r#"{"Version":1,"Machine":"pc","Created":"2026-01-02T03:04:05Z","Changes":[{"Id":"x","Before":{"Exists":false,"Value":0},"Desired":1,"Key":"HKLM"}]}"#,
        ] {
            assert!(
                serde_json::from_str::<PrivacyBackup>(bad).is_err(),
                "{bad} must be rejected"
            );
        }
    }
}

/// Counts of audit states, for summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuditSummary {
    /// Policies whose stored value equals the desired value.
    pub configured: usize,
    /// Policies that are absent or different.
    pub different: usize,
    /// Policies that could not be read.
    pub unknown: usize,
}

impl AuditSummary {
    /// Tallies a list of findings.
    pub fn of(findings: &[AuditFinding]) -> AuditSummary {
        findings
            .iter()
            .fold(AuditSummary::default(), |mut summary, finding| {
                match finding.state {
                    AuditState::Configured => summary.configured += 1,
                    AuditState::DifferentOrNotConfigured => summary.different += 1,
                    AuditState::Unknown => summary.unknown += 1,
                }
                summary
            })
    }

    /// Total number of findings.
    pub fn total(&self) -> usize {
        self.configured + self.different + self.unknown
    }
}

#[cfg(test)]
mod summary_tests {
    use super::*;

    #[test]
    fn summary_counts_each_state() {
        let finding = |state| AuditFinding {
            id: "x",
            state,
            current: None,
            desired: 1,
            impact: "",
            error: None,
        };
        let findings = [
            finding(AuditState::Configured),
            finding(AuditState::Unknown),
            finding(AuditState::DifferentOrNotConfigured),
            finding(AuditState::Configured),
        ];
        let summary = AuditSummary::of(&findings);
        assert_eq!(
            summary,
            AuditSummary {
                configured: 2,
                different: 1,
                unknown: 1
            }
        );
        assert_eq!(summary.total(), 4);
        assert_eq!(AuditSummary::of(&[]).total(), 0);
    }
}
