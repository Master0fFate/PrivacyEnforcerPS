# Changelog

All notable changes to Privacy Enforcer are described here, newest first.

## 1.0.0-rc.1 — 2026-10-21

The Rust rewrite. The C#/.NET application it replaces now lives under `BACKUP/old/csharp-2025`.

### What's new

- **A desktop app.** `pef-gui` is a native window built on GPUI Fast with a design in the
  spirit of Vercel's Geist system: Audit, Policies, Enforce, Restore and About screens, light
  and dark palettes, keyboard shortcuts, and a confirmation dialog that still makes you type
  `ENFORCE` or `RESTORE`. On a machine without a supported Windows registry it opens in a
  clearly labelled simulation so you can explore it without changing anything.
- **A much larger catalog.** 65 documented policies across ten areas instead of seven:
  diagnostic data, personalization and ads, Search, input and speech, location, app
  permissions, activity and sync, network services, Windows AI (Recall and Click to Do) and
  Microsoft Edge. Every entry still maps to one `DWORD` under `HKEY_LOCAL_MACHINE`, carries
  an impact statement and a Microsoft reference, and is reversible.
- **Presets as shorthands.** `--preset baseline`, `app-permissions`, `edge` and `strict`
  expand to an explicit list that the preview shows in full. They never skip the preview or
  the typed confirmation, and nothing is ever selected on your behalf.
- **A friendlier command line.** `pef` keeps the audit-first grammar (`list`, `audit`,
  `enforce`, `restore`, `help`) and adds `version`, `list --presets`, category grouping in
  `list`, an audit summary line, and clearer messages that say whether anything was written.
- **Compatible backups.** Recovery files use the same JSON layout as the previous release,
  so a backup made by the .NET version restores with this one.

### What stayed the same on purpose

- Audit and preview never write and never ask for elevation.
- The complete plan is saved to a new file, flushed and read back before the first registry
  write; every write is read back; a value changed by another program stops the operation.
- Restore is idempotent, walks the plan in reverse and tolerates a partially applied plan.
- The tool does not touch Microsoft Defender, Windows Update, User Account Control, event
  logs, scheduled tasks, services, authentication or network connectivity.

### For contributors

- The workspace is split into `pef-core` (pure logic, no platform access), `pef-platform`
  (registry, OS gate, elevation), `pef-cli` and `pef-gui`.
- The test suite covers the engine rules with an in-memory store, the backup format, the
  argument grammar, the command line end to end with a fake host, the desktop session model,
  the theme's contrast ratios, and the views themselves through GPUI's headless test context.
- CI runs on Linux and Windows and type-checks the Windows target from Linux; tagged pushes
  build release archives for both platforms.
