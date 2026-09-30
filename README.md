# PrivacyEnforcerPro

An audit-first Windows privacy policy tool. Despite the historical repository name,
the maintained application is C#/.NET, not the archived PowerShell scripts.

**This is a safety-focused redesign.** The old immediate-execution menu is replaced
with explicit policy selection, previews, backup-before-write and conflict-aware
restore. The old service, firewall, cleanup and log-purging modules are retained as
legacy source only and are not exposed by the application. Do not run `BACKUP/`.

## Requirements

- A currently serviced Windows 11 Pro, Enterprise or Education installation
- .NET 10 SDK to build (or .NET 10 runtime for framework-dependent binaries)
- Normal terminal for audit; administrator terminal only for applying/restoring
- Windows Home, Server, Windows 10 and non-Windows hosts are rejected for policy
  access. `list` and `help` work anywhere. The OS gate checks build 22000+ and an
  explicit edition allowlist; it does not determine your edition's servicing date.

.NET 10 is the LTS target. Check Microsoft's [.NET support policy](https://dotnet.microsoft.com/en-us/platform/support/policy/dotnet-core)
and [Windows release health](https://learn.microsoft.com/en-us/windows/release-health/)
for runtime and operating-system servicing requirements.

## Start with an audit

```powershell
dotnet build PrivacyEnforcerPro.sln -c Release
dotnet run --project PrivacyEnforcerPro/PrivacyEnforcerPro.UI -c Release --no-build -- list
dotnet run --project PrivacyEnforcerPro/PrivacyEnforcerPro.UI -c Release --no-build -- audit
dotnet run --project PrivacyEnforcerPro/PrivacyEnforcerPro.UI -c Release --no-build -- audit --json
```

No arguments means `audit`. Audit does not create logs, request elevation, send
network requests, or modify registry values. `Unknown` indicates an unreadable or
unexpected value, not successful protection. `Configured` only confirms the stored
registry value, not effective enforcement or protection from every app.

## Preview, then apply selected policies

Available IDs: `advertising-id`, `clipboard-sync`, `activity-feed`, `online-speech`,
`app-camera`, `app-microphone`, `app-location`. Run `list` for the exact registry
value, desired value, impact and Microsoft policy reference for every option.
No policy is automatically selected for enforcement and there is no blanket preset.

```powershell
# Read-only preview, no administrator requirement
dotnet run --project PrivacyEnforcerPro/PrivacyEnforcerPro.UI -c Release --no-build -- enforce --select advertising-id,clipboard-sync

# In an administrator terminal, using an existing PRIVATE local folder:
dotnet run --project PrivacyEnforcerPro/PrivacyEnforcerPro.UI -c Release --no-build -- enforce --select advertising-id,clipboard-sync --apply --backup C:\PrivateBackups\privacy-before.json
```

Inspect the preview and type `ENFORCE` at the prompt. Redirected stdin cannot approve
changes. The backup must be a new file and its parent directory must already exist.
Protect that folder with your normal user/admin permissions; avoid public shares,
cloud-synced folders and untrusted backup files. It contains the computer name and
previous values, not credentials. The tool does not change folder permissions.

The complete plan is saved and flushed before the first policy write. Missing
values and explicit DWORD values are distinguished. Each write is read back.
Repeated enforcement does nothing if the selected values already match.

## Restore

```powershell
# Validate and preview only
dotnet run --project PrivacyEnforcerPro/PrivacyEnforcerPro.UI -c Release --no-build -- restore --backup C:\PrivateBackups\privacy-before.json

# In an administrator terminal; review, then type RESTORE
dotnet run --project PrivacyEnforcerPro/PrivacyEnforcerPro.UI -c Release --no-build -- restore --backup C:\PrivateBackups\privacy-before.json --apply
```

Restore validates version, machine name, known IDs, duplicate entries and desired
values. It never accepts registry paths from the file. It restores the original
DWORD or removes only a value that was absent. Shared keys/other values stay intact.
Backups are not signed or encrypted: only restore your own trusted original file.
Machine-name matching is a guard against mistakes, not cryptographic identity.

Apply and restore stop on errors. They are not atomic transactions: a power loss,
permission failure or concurrent writer may leave a partial operation. Keep the
backup, audit, then preview restore. Restore handles partially applied and already
restored entries. It refuses to overwrite a value changed to something unexpected
by another program or administrator. There is deliberately no force bypass. There
is a small race between checking a value and writing it; do not run concurrent
policy tools. Restore backups in reverse chronological order.

## Boundaries and risks

- These are machine-wide policy settings; confirm every selected impact first
- Camera/microphone/location restrictions can disrupt calls, recording and maps;
  cloud speech restrictions can disrupt dictation
- Restart affected applications; sign-out/restart may be required for some policies
- On managed devices, consult your administrator; MDM/Group Policy can override or
  reapply values. App-specific exceptions and desktop-app behavior need review
- This tool does not guarantee anonymity, eliminate all telemetry, clear past cloud
  data, or enforce per-app firewall isolation
- It does not disable Defender, Windows Update, UAC, event logs, scheduled tasks,
  services, authentication or network connectivity
- It does not auto-update, install persistence, download blocklists, phone home,
  collect secrets, or execute shell commands in the new policy workflow

Policy mappings were checked against [Microsoft's Privacy policy reference](https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy).

## Tests and validation

```powershell
dotnet test PrivacyEnforcerPro.sln -c Release
```

The suite uses an in-memory registry substitute and temporary JSON files. It tests
read-only defaults, dry-run, catalog selection, backup failures, read-back failures,
partial recovery, conflicts, malformed backups, idempotence and argument validation.
CI builds/tests on Linux and Windows and runs the read-only catalog. It never
applies policies to runners. Pester is not needed for the C# application.

Native registry writes, Windows policy propagation, reboot/sign-out behavior and
app compatibility still require an authorized disposable Windows 11 VM test. The
mock suite and Windows CI alone do **not** establish those results. Before a release,
use the [manual validation checklist](docs/WINDOWS-VALIDATION.md).

Exit codes: `0` successful read/preview/write or user cancellation; `1` invalid
input, unsupported platform or stopped operation; `2` audit contains unknowns.
