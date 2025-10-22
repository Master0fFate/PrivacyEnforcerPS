# PrivacyEnforcerPro

A Windows-focused privacy hardening toolkit with a modular console UI.

## Features
- File cleanup and log purging
- Telemetry suppression for common services
- Browser cache and cookie hygiene
- Service and startup management
- Network guard and basic system hardening
- Diagnostics to inspect current system state

## Quick Start
- Requirements: Windows, PowerShell, .NET 8 SDK (for source), admin privileges recommended
- Run from source: `dotnet run --project PrivacyEnforcerPro/PrivacyEnforcerPro.UI`
- Download: Use Releases for prebuilt Windows binaries

## Notes
- Some operations require elevation to manage services and registry.
- Designed for Windows. Platform analysis warnings are expected and safe on Windows.