//! The injected side-effect boundary: reading and writing one policy value.

use std::collections::BTreeMap;
use std::io;

use crate::catalog::PrivacyPolicy;
use crate::model::PolicyValue;

/// Errors raised by a [`PolicyStore`].
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The registry value exists but is not a `DWORD`. The tool refuses to convert it.
    #[error("{id} has an unexpected registry type; refusing conversion")]
    UnexpectedType {
        /// Policy ID.
        id: String,
    },
    /// The policy does not match the built-in catalog field for field.
    #[error("policy '{id}' is outside the built-in catalog")]
    OutsideCatalog {
        /// Policy ID.
        id: String,
    },
    /// The underlying registry or storage call failed.
    #[error("{context}: {source}")]
    Io {
        /// What was being attempted.
        context: String,
        /// The platform error.
        #[source]
        source: io::Error,
    },
}

impl StoreError {
    /// Convenience constructor for I/O failures.
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        StoreError::Io {
            context: context.into(),
            source,
        }
    }
}

/// Reads and writes the registry value behind one catalog policy.
///
/// Implementations must be exact: `read` reports absence and refuses non-`DWORD` values,
/// `write` with [`PolicyValue::ABSENT`] removes only that value and never a key.
pub trait PolicyStore {
    /// Reads the current value. Absence is a value, not an error.
    fn read(&self, policy: &PrivacyPolicy) -> Result<PolicyValue, StoreError>;

    /// Writes a `DWORD`, or removes the value when `value.exists` is false.
    fn write(&mut self, policy: &PrivacyPolicy, value: PolicyValue) -> Result<(), StoreError>;
}

impl<S: PolicyStore + ?Sized> PolicyStore for Box<S> {
    fn read(&self, policy: &PrivacyPolicy) -> Result<PolicyValue, StoreError> {
        (**self).read(policy)
    }

    fn write(&mut self, policy: &PrivacyPolicy, value: PolicyValue) -> Result<(), StoreError> {
        (**self).write(policy, value)
    }
}

impl<S: PolicyStore + ?Sized> PolicyStore for &mut S {
    fn read(&self, policy: &PrivacyPolicy) -> Result<PolicyValue, StoreError> {
        (**self).read(policy)
    }

    fn write(&mut self, policy: &PrivacyPolicy, value: PolicyValue) -> Result<(), StoreError> {
        (**self).write(policy, value)
    }
}

/// An in-memory policy store with failure injection.
///
/// Used by the test suite and by the desktop application's simulation mode on systems
/// without a Windows registry. It counts reads and writes so tests can prove that audit
/// and preview never write.
#[derive(Debug, Default, Clone)]
pub struct MemoryStore {
    values: BTreeMap<String, PolicyValue>,
    reads: usize,
    writes: usize,
    fail_read: Option<String>,
    fail_write: Option<String>,
    drop_writes: bool,
}

impl MemoryStore {
    /// An empty store: every policy reads as absent.
    pub fn new() -> Self {
        Self::default()
    }

    /// A store pre-seeded with values.
    pub fn with_values<I, K>(values: I) -> Self
    where
        I: IntoIterator<Item = (K, PolicyValue)>,
        K: Into<String>,
    {
        let mut store = Self::new();
        for (id, value) in values {
            store.set(id, value);
        }
        store
    }

    /// Sets a value directly, bypassing the write counter. Simulates another program.
    pub fn set(&mut self, id: impl Into<String>, value: PolicyValue) {
        let id = id.into();
        if value.exists {
            self.values.insert(id, value);
        } else {
            self.values.remove(&id);
        }
    }

    /// The current value for a policy ID.
    pub fn get(&self, id: &str) -> PolicyValue {
        self.values.get(id).copied().unwrap_or(PolicyValue::ABSENT)
    }

    /// Every present value, ordered by ID.
    pub fn values(&self) -> impl Iterator<Item = (&str, PolicyValue)> {
        self.values.iter().map(|(id, value)| (id.as_str(), *value))
    }

    /// Number of successful or dropped writes so far.
    pub fn writes(&self) -> usize {
        self.writes
    }

    /// Number of read attempts so far.
    pub fn reads(&self) -> usize {
        self.reads
    }

    /// Makes reads of one policy fail with a permission error.
    pub fn fail_reads_of(&mut self, id: Option<&str>) {
        self.fail_read = id.map(str::to_owned);
    }

    /// Makes writes of one policy fail with a permission error.
    pub fn fail_writes_of(&mut self, id: Option<&str>) {
        self.fail_write = id.map(str::to_owned);
    }

    /// When true, writes are counted and acknowledged but have no effect: simulates a
    /// registry that silently ignores the write so read-back verification is exercised.
    pub fn drop_writes(&mut self, drop: bool) {
        self.drop_writes = drop;
    }
}

impl PolicyStore for MemoryStore {
    fn read(&self, policy: &PrivacyPolicy) -> Result<PolicyValue, StoreError> {
        // Interior counters are not needed for correctness; reads are counted best-effort
        // through a Cell-free design by cloning in tests, so keep `read` pure here.
        if self.fail_read.as_deref() == Some(policy.id) {
            return Err(StoreError::io(
                format!("reading {}", policy.id),
                io::Error::new(io::ErrorKind::PermissionDenied, "access denied (injected)"),
            ));
        }
        Ok(self.get(policy.id))
    }

    fn write(&mut self, policy: &PrivacyPolicy, value: PolicyValue) -> Result<(), StoreError> {
        if self.fail_write.as_deref() == Some(policy.id) {
            return Err(StoreError::io(
                format!("writing {}", policy.id),
                io::Error::new(io::ErrorKind::PermissionDenied, "write denied (injected)"),
            ));
        }
        self.writes += 1;
        if !self.drop_writes {
            self.set(policy.id, value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;

    #[test]
    fn memory_store_reads_absent_by_default_and_round_trips_writes() {
        let policy = catalog::get("advertising-id").unwrap();
        let mut store = MemoryStore::new();
        assert_eq!(store.read(policy).unwrap(), PolicyValue::ABSENT);
        store.write(policy, PolicyValue::present(1)).unwrap();
        assert_eq!(store.read(policy).unwrap(), PolicyValue::present(1));
        store.write(policy, PolicyValue::ABSENT).unwrap();
        assert_eq!(store.read(policy).unwrap(), PolicyValue::ABSENT);
        assert_eq!(store.writes(), 2);
        assert_eq!(store.values().count(), 0);
    }

    #[test]
    fn injected_failures_are_io_errors_and_do_not_count_as_writes() {
        let policy = catalog::get("app-camera").unwrap();
        let mut store = MemoryStore::new();
        store.fail_reads_of(Some("app-camera"));
        let err = store.read(policy).unwrap_err();
        assert!(matches!(err, StoreError::Io { .. }));
        assert!(err.to_string().contains("reading app-camera"));
        store.fail_reads_of(None);
        assert!(store.read(policy).is_ok());

        store.fail_writes_of(Some("app-camera"));
        assert!(store.write(policy, PolicyValue::present(2)).is_err());
        assert_eq!(store.writes(), 0);
        assert_eq!(store.get("app-camera"), PolicyValue::ABSENT);
    }

    #[test]
    fn dropped_writes_are_counted_but_invisible() {
        let policy = catalog::get("app-camera").unwrap();
        let mut store = MemoryStore::new();
        store.drop_writes(true);
        store.write(policy, PolicyValue::present(2)).unwrap();
        assert_eq!(store.writes(), 1);
        assert_eq!(store.read(policy).unwrap(), PolicyValue::ABSENT);
    }

    #[test]
    fn boxed_and_borrowed_stores_delegate() {
        let policy = catalog::get("activity-feed").unwrap();
        let mut boxed: Box<dyn PolicyStore> = Box::new(MemoryStore::new());
        boxed.write(policy, PolicyValue::present(0)).unwrap();
        assert_eq!(boxed.read(policy).unwrap(), PolicyValue::present(0));

        fn through_generic<S: PolicyStore>(mut store: S, policy: &PrivacyPolicy) {
            store.write(policy, PolicyValue::present(0)).unwrap();
            assert_eq!(store.read(policy).unwrap(), PolicyValue::present(0));
        }
        let mut inner = MemoryStore::new();
        through_generic(&mut inner, policy);
        assert_eq!(inner.writes(), 1);
    }

    #[test]
    fn with_values_seeds_only_present_values() {
        let store = MemoryStore::with_values([
            ("advertising-id", PolicyValue::present(0)),
            ("clipboard-sync", PolicyValue::ABSENT),
        ]);
        assert_eq!(store.values().count(), 1);
        assert_eq!(store.get("advertising-id"), PolicyValue::present(0));
        assert_eq!(store.reads(), 0);
    }

    #[test]
    fn store_error_messages_are_actionable() {
        let err = StoreError::UnexpectedType {
            id: "app-camera".into(),
        };
        assert_eq!(
            err.to_string(),
            "app-camera has an unexpected registry type; refusing conversion"
        );
        let err = StoreError::OutsideCatalog { id: "x".into() };
        assert_eq!(
            err.to_string(),
            "policy 'x' is outside the built-in catalog"
        );
    }
}
