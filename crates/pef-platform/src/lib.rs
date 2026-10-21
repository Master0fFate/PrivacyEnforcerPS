//! Platform access for Privacy Enforcer: the Windows registry policy store, the
//! operating-system gate, the elevation check and the machine name.
//!
//! Everything here is the thin, non-portable layer below [`pef_core`]. On systems other
//! than Windows every function fails closed: the catalog and the argument grammar still
//! work, policy access does not.

#![deny(unsafe_code)]

use pef_core::PolicyStore;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::WindowsPolicyStore;

/// The first Windows 11 build.
pub const MINIMUM_BUILD: u32 = 22_000;

/// Windows editions whose policy behaviour this tool is validated against.
pub const SUPPORTED_EDITIONS: [&str; 9] = [
    "Professional",
    "ProfessionalN",
    "ProfessionalEducation",
    "ProfessionalWorkstation",
    "Enterprise",
    "EnterpriseN",
    "EnterpriseS",
    "Education",
    "EducationN",
];

/// Why policy access is refused on this system.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlatformError {
    /// Not Windows, or a Windows build before Windows 11.
    #[error(
        "Requires Windows 11 (build 22000+) with a currently serviced Pro, Enterprise or Education edition. Windows 10, Server and Home are not supported. list and help work anywhere."
    )]
    UnsupportedOperatingSystem,
    /// Windows 11, but an edition outside [`SUPPORTED_EDITIONS`].
    #[error("Unsupported Windows edition: {edition}. No changes made.")]
    UnsupportedEdition {
        /// The `EditionID` that was found.
        edition: String,
    },
    /// The version information could not be read.
    #[error("Could not determine the Windows version: {0}")]
    Unreadable(String),
}

/// Version facts read from `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsVersion {
    /// `CurrentMajorVersionNumber`; 10 for both Windows 10 and Windows 11.
    pub major: u32,
    /// `CurrentBuildNumber`; 22000 and above is Windows 11.
    pub build: u32,
    /// `EditionID`, for example `Professional`.
    pub edition: String,
    /// `DisplayVersion`, for example `24H2`. Empty when absent.
    pub display_version: String,
}

impl WindowsVersion {
    /// Checks the build and edition against what the tool supports.
    pub fn check_supported(&self) -> Result<(), PlatformError> {
        if self.major < 10 || self.build < MINIMUM_BUILD {
            return Err(PlatformError::UnsupportedOperatingSystem);
        }
        if !is_supported_edition(&self.edition) {
            return Err(PlatformError::UnsupportedEdition {
                edition: self.edition.clone(),
            });
        }
        Ok(())
    }

    /// A one-line description for reports.
    pub fn describe(&self) -> String {
        let family = if self.build >= MINIMUM_BUILD {
            "Windows 11"
        } else {
            "Windows 10"
        };
        if self.display_version.is_empty() {
            format!("{family} {} (build {})", self.edition, self.build)
        } else {
            format!(
                "{family} {} {} (build {})",
                self.edition, self.display_version, self.build
            )
        }
    }
}

/// True for the exact `EditionID` strings in [`SUPPORTED_EDITIONS`].
pub fn is_supported_edition(edition: &str) -> bool {
    SUPPORTED_EDITIONS.contains(&edition)
}

/// Reads the Windows version. Fails closed on other systems.
pub fn windows_version() -> Result<WindowsVersion, PlatformError> {
    #[cfg(windows)]
    {
        windows::version()
    }
    #[cfg(not(windows))]
    {
        Err(PlatformError::UnsupportedOperatingSystem)
    }
}

/// Reads the Windows version and checks it, returning it for display.
pub fn require_supported_windows() -> Result<WindowsVersion, PlatformError> {
    let version = windows_version()?;
    version.check_supported()?;
    Ok(version)
}

/// True when the process token is elevated (the "administrator terminal" case).
pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        windows::is_elevated()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// The computer name recorded in backups. On Windows this is `COMPUTERNAME`, the same
/// value the earlier .NET release used, so its backups still match.
pub fn machine_name() -> String {
    for variable in ["COMPUTERNAME", "HOSTNAME"] {
        if let Ok(name) = std::env::var(variable)
            && !name.trim().is_empty()
        {
            return name.trim().to_owned();
        }
    }
    if let Ok(name) = std::fs::read_to_string("/etc/hostname")
        && !name.trim().is_empty()
    {
        return name.trim().to_owned();
    }
    "unknown-machine".to_owned()
}

/// Opens the registry-backed policy store after the operating-system gate passes.
pub fn open_policy_store() -> Result<Box<dyn PolicyStore>, PlatformError> {
    require_supported_windows()?;
    #[cfg(windows)]
    {
        Ok(Box::new(WindowsPolicyStore))
    }
    #[cfg(not(windows))]
    {
        Err(PlatformError::UnsupportedOperatingSystem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(major: u32, build: u32, edition: &str) -> WindowsVersion {
        WindowsVersion {
            major,
            build,
            edition: edition.into(),
            display_version: String::new(),
        }
    }

    #[test]
    fn supported_editions_are_exact_matches() {
        for edition in SUPPORTED_EDITIONS {
            assert!(is_supported_edition(edition));
        }
        for edition in [
            "Core",
            "CoreSingleLanguage",
            "ServerStandard",
            "professional",
            "",
            "Pro",
        ] {
            assert!(!is_supported_edition(edition), "{edition}");
        }
    }

    #[test]
    fn build_and_edition_gate() {
        assert_eq!(
            version(10, 22_000, "Professional").check_supported(),
            Ok(())
        );
        assert_eq!(version(10, 26_100, "EnterpriseS").check_supported(), Ok(()));
        assert_eq!(
            version(10, 19_045, "Professional").check_supported(),
            Err(PlatformError::UnsupportedOperatingSystem)
        );
        assert_eq!(
            version(6, 99_999, "Professional").check_supported(),
            Err(PlatformError::UnsupportedOperatingSystem)
        );
        assert_eq!(
            version(10, 26_100, "Core").check_supported(),
            Err(PlatformError::UnsupportedEdition {
                edition: "Core".into()
            })
        );
        assert_eq!(
            version(10, 26_100, "ServerStandard")
                .check_supported()
                .unwrap_err()
                .to_string(),
            "Unsupported Windows edition: ServerStandard. No changes made."
        );
    }

    #[test]
    fn descriptions_are_readable() {
        assert_eq!(
            version(10, 19_045, "Professional").describe(),
            "Windows 10 Professional (build 19045)"
        );
        let mut v = version(10, 26_100, "Professional");
        v.display_version = "24H2".into();
        assert_eq!(v.describe(), "Windows 11 Professional 24H2 (build 26100)");
    }

    #[test]
    fn machine_name_is_never_empty() {
        assert!(!machine_name().is_empty());
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_fails_closed() {
        assert_eq!(
            windows_version(),
            Err(PlatformError::UnsupportedOperatingSystem)
        );
        assert_eq!(
            require_supported_windows().unwrap_err(),
            PlatformError::UnsupportedOperatingSystem
        );
        assert!(open_policy_store().is_err());
        assert!(!is_elevated());
        assert!(
            PlatformError::UnsupportedOperatingSystem
                .to_string()
                .contains("Windows 11")
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_version_is_readable() {
        let version = windows_version().expect("version readable on Windows");
        assert_eq!(version.major, 10);
        assert!(version.build > 10_000);
        assert!(!version.edition.is_empty());
        let _ = is_elevated();
    }
}
