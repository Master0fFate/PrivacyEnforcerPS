# Windows validation before a stable release

Use a disposable, snapshotted, currently supported Windows 11 Pro VM. Get explicit approval
before changing that machine. This checklist is not executed by CI: the mock suite and the
Windows build job prove the logic and the packaging, not the behaviour of real policies.

1. As a standard user run `pef list`, `pef audit` and `pef enforce --preset baseline`
   (preview). Open `pef-gui` and visit every screen. Confirm no file, registry or service
   changes and no elevation prompt.
2. Capture original catalog values independently with Registry Editor. Include values that
   are absent and at least one explicit `DWORD` zero. Do not use production settings.
3. In an elevated terminal preview one policy, start the apply, cancel at the prompt, and
   confirm nothing changed. Pipe input into the command and confirm it refuses to apply.
   Repeat with the desktop app's dialog: a wrong word must be rejected.
4. Apply one policy with a new backup in a private local folder. Verify the backup contents
   and the exact registry value independently. Confirm every other catalog value and all
   security and service settings are unchanged.
5. Re-run the same enforcement: expect no writes and no new backup. Preview the restore,
   then restore. Verify the original presence, type and value; restore again safely.
6. Test an unwritable backup folder, an existing backup filename, a backup from another
   machine and invalid JSON: all must stop before any registry write. Create a non-`DWORD`
   value at a catalog location and confirm the audit reports it as Unknown and the plan
   refuses it.
7. In the VM, change one applied value by hand; restore must report a conflict rather than
   overwrite it. Put it back; restore must then succeed and skip entries already restored.
8. With explicit permission test each high-impact policy separately: location platform,
   cloud notifications, online speech, app camera, microphone, location, background apps
   and programmatic screen capture. Restart apps; verify behaviour and reversal. Record
   per-app exceptions, desktop-app limits and sign-out or restart needs.
9. Verify Windows Update, Microsoft Defender, event logging, authentication and connectivity
   remain functional. Do not infer this from registry assertions alone.
10. Test the refusal on Home, Windows Server and Windows 10; test the exact editions and
    builds planned for release, including the Edge policies with Edge installed.

Record the OS edition and build, app versions, selected IDs, expected and actual behaviour
and remaining limitations. Never publish machine identifiers or private backup files.
