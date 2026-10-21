# Windows validation before release

Use a disposable, snapshotted, currently supported Windows 11 Pro VM. Obtain
explicit approval before changing that machine. This checklist has not been
executed as part of the Linux implementation pass.

1. As a standard user run `list`, default audit and selected enforce preview.
   Confirm no file/registry/service changes and no elevation prompt.
2. Capture original catalog values and types independently with Registry Editor.
   Exercise absent values and explicit DWORD zero. Do not use production settings.
3. In an elevated terminal preview one selected policy, cancel its prompt, and
   confirm no changes. Verify redirected stdin cannot confirm.
4. Apply one policy with a new backup in an admin-controlled local directory.
   Independently verify backup contents and exact registry value. Confirm the
   other six catalog values and all security/service settings are unchanged.
5. Re-run selected enforcement: expect no writes or new backup. Preview restore,
   then restore. Verify original presence/type/value; repeat restore safely.
6. Test an unwritable backup folder, an existing backup filename and invalid JSON:
   all must stop before registry writes. Test unsupported registry types.
7. Simulate an interrupted apply in the test harness (not by corrupting the VM).
   Review partial recovery tests. In the VM change one test value independently;
   restore must report a conflict rather than overwrite it.
8. With explicit permission test each high-impact policy separately. Restart apps;
   verify camera, microphone, location and speech behavior and reversal. Document
   per-app exceptions, desktop-app limits and policy refresh/sign-out needs.
9. Verify Windows Update, Defender, event logging, authentication and connectivity
   remain functional. Do not infer this solely from registry-value assertions.
10. Test read-only rejection on Home, Windows Server and unsupported systems;
    test the exact Windows editions/builds planned for release.

Record OS edition/build, app versions, selected IDs, expected/actual behavior and
remaining limitations. Never publish machine identifiers or private backup files.
