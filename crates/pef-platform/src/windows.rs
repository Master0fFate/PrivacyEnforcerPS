//! Windows implementation: registry reads and writes through the 64-bit view, the
//! version keys and the token elevation check.
//!
//! This is the only module in the workspace that uses `unsafe`, for two Win32 calls:
//! flushing a registry key after a write and querying the process token.

#![allow(unsafe_code)]

use std::io;

use pef_core::{PolicyStore, PolicyValue, PrivacyPolicy, StoreError, catalog};
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, KEY_WOW64_64KEY, RegType};
use winreg::{RegKey, RegValue};

use crate::{PlatformError, WindowsVersion};

const CURRENT_VERSION: &str = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";

/// Reads and writes catalog policies in `HKEY_LOCAL_MACHINE` (64-bit view).
///
/// Every call first checks that the policy is the catalog's own entry, so a forged
/// policy with a different key or value can never reach the registry.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsPolicyStore;

fn check_catalog(policy: &PrivacyPolicy) -> Result<(), StoreError> {
    if catalog::contains(policy) {
        Ok(())
    } else {
        Err(StoreError::OutsideCatalog {
            id: policy.id.to_owned(),
        })
    }
}

fn flush(key: &RegKey, policy: &PrivacyPolicy) -> Result<(), StoreError> {
    // SAFETY: `raw_handle` is a valid, open HKEY owned by `key` for the duration of
    // the call, and RegFlushKey takes no other pointers.
    let status = unsafe {
        windows_sys::Win32::System::Registry::RegFlushKey(
            key.raw_handle() as windows_sys::Win32::System::Registry::HKEY
        )
    };
    if status == windows_sys::Win32::Foundation::ERROR_SUCCESS {
        Ok(())
    } else {
        Err(StoreError::io(
            format!("flushing {}", policy.id),
            io::Error::from_raw_os_error(status as i32),
        ))
    }
}

impl PolicyStore for WindowsPolicyStore {
    fn read(&self, policy: &PrivacyPolicy) -> Result<PolicyValue, StoreError> {
        check_catalog(policy)?;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = match hklm.open_subkey_with_flags(policy.key, KEY_READ | KEY_WOW64_64KEY) {
            Ok(key) => key,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(PolicyValue::ABSENT),
            Err(err) => return Err(StoreError::io(format!("opening HKLM\\{}", policy.key), err)),
        };
        match key.get_raw_value(policy.name) {
            Ok(RegValue {
                vtype: RegType::REG_DWORD,
                bytes,
            }) if bytes.len() == 4 => Ok(PolicyValue::present(u32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]))),
            Ok(_) => Err(StoreError::UnexpectedType {
                id: policy.id.to_owned(),
            }),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(PolicyValue::ABSENT),
            Err(err) => Err(StoreError::io(
                format!("reading {}", policy.registry_path()),
                err,
            )),
        }
    }

    fn write(&mut self, policy: &PrivacyPolicy, value: PolicyValue) -> Result<(), StoreError> {
        check_catalog(policy)?;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if value.exists {
            let (key, _) = hklm
                .create_subkey_with_flags(policy.key, KEY_SET_VALUE | KEY_WOW64_64KEY)
                .map_err(|err| StoreError::io(format!("creating HKLM\\{}", policy.key), err))?;
            key.set_value(policy.name, &value.value).map_err(|err| {
                StoreError::io(format!("writing {}", policy.registry_path()), err)
            })?;
            flush(&key, policy)
        } else {
            // Remove only this value. Shared keys and other values stay intact.
            let key = match hklm.open_subkey_with_flags(policy.key, KEY_SET_VALUE | KEY_WOW64_64KEY)
            {
                Ok(key) => key,
                Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
                Err(err) => {
                    return Err(StoreError::io(format!("opening HKLM\\{}", policy.key), err));
                }
            };
            match key.delete_value(policy.name) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(StoreError::io(
                        format!("removing {}", policy.registry_path()),
                        err,
                    ));
                }
            }
            flush(&key, policy)
        }
    }
}

/// Reads the version keys that Windows maintains under `CurrentVersion`.
pub(crate) fn version() -> Result<WindowsVersion, PlatformError> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(CURRENT_VERSION, KEY_READ | KEY_WOW64_64KEY)
        .map_err(|err| PlatformError::Unreadable(err.to_string()))?;
    let major: u32 = key
        .get_value("CurrentMajorVersionNumber")
        .map_err(|err| PlatformError::Unreadable(format!("CurrentMajorVersionNumber: {err}")))?;
    let build: String = key
        .get_value("CurrentBuildNumber")
        .map_err(|err| PlatformError::Unreadable(format!("CurrentBuildNumber: {err}")))?;
    let build: u32 = build
        .trim()
        .parse()
        .map_err(|_| PlatformError::Unreadable(format!("CurrentBuildNumber is '{build}'")))?;
    let edition: String = key
        .get_value("EditionID")
        .map_err(|err| PlatformError::Unreadable(format!("EditionID: {err}")))?;
    let display_version: String = key.get_value("DisplayVersion").unwrap_or_default();
    Ok(WindowsVersion {
        major,
        build,
        edition,
        display_version,
    })
}

/// True when the current process token is elevated.
pub(crate) fn is_elevated() -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::{
        GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token = std::ptr::null_mut();
    // SAFETY: GetCurrentProcess returns a pseudo handle that needs no closing;
    // `token` is a valid out-pointer and is closed below when opened.
    let opened = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) };
    if opened == 0 {
        return false;
    }
    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut returned = 0u32;
    // SAFETY: `elevation` is a properly sized, writable TOKEN_ELEVATION and the size
    // passed matches it; `returned` is a valid out-pointer.
    let queried = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
    };
    // SAFETY: `token` was opened above and is closed exactly once.
    unsafe {
        CloseHandle(token);
    }
    queried != 0 && elevation.TokenIsElevated != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forged_policies_never_reach_the_registry() {
        let real = catalog::get("advertising-id").unwrap();
        let forged = PrivacyPolicy {
            key: "SOFTWARE\\Policies\\Microsoft\\Windows\\Evil",
            ..*real
        };
        let mut store = WindowsPolicyStore;
        assert!(matches!(
            store.read(&forged),
            Err(StoreError::OutsideCatalog { .. })
        ));
        assert!(matches!(
            store.write(&forged, PolicyValue::present(1)),
            Err(StoreError::OutsideCatalog { .. })
        ));
    }

    #[test]
    fn reading_the_catalog_never_panics() {
        // Read-only: absent, present or an unexpected type are all acceptable answers.
        let store = WindowsPolicyStore;
        for policy in catalog::all() {
            match store.read(policy) {
                Ok(_) | Err(StoreError::UnexpectedType { .. }) => {}
                Err(err) => panic!("{}: {err}", policy.id),
            }
        }
    }

    #[test]
    fn version_keys_are_present() {
        let version = version().unwrap();
        assert_eq!(version.major, 10);
        assert!(version.build >= 10_240);
    }
}
