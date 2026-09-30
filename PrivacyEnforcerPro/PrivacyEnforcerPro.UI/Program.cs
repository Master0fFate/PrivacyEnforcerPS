using System.Text.Json;
using System.Security.Principal;
using PrivacyEnforcerPro.Core.Policies;
using PrivacyEnforcerPro.Infrastructure.Services;

// Audit is the default. No logs/directories/elevation or service startup on launch.
try
{
    var options = CommandOptions.Parse(args);
    if (options.Command == "help")
    {
        Console.WriteLine("PrivacyEnforcerPro: audit-first Windows privacy policies\n" +
            "  list [--json]\n  audit [--select ID,ID] [--json] (default)\n" +
            "  enforce --select ID,ID [--apply --backup NEW_FILE.json]\n" +
            "  restore --backup FILE.json [--apply]\n" +
            "Without --apply, enforce and restore only preview. Apply requires administrator privileges and a typed confirmation.\n" +
            "Use a private local backup path. Review each policy's impact with list. No blanket/all profile.");
        return 0;
    }
    var policies = options.Ids.Length == 0 ? PolicyCatalog.All : PolicyCatalog.Select(options.Ids);
    if (options.Command == "list")
    {
        if (options.Json) Console.WriteLine(JsonSerializer.Serialize(policies, PrivacyBackupFile.Json));
        else foreach (var p in policies) Console.WriteLine($"{p.Id}: HKLM\\{p.Key}\\{p.Name} = {p.Desired}\n  Impact: {p.Impact}\n  Source: {p.Source}");
        return 0;
    }
    if (!OperatingSystem.IsWindowsVersionAtLeast(10, 0, 22000))
        throw new PlatformNotSupportedException("Requires Windows 11 (build 22000+) with a currently serviced Pro, Enterprise or Education edition. Windows 10, Server and Home are not supported. list/help are platform independent.");
    var edition = WindowsPolicyStore.Edition();
    if (!WindowsPolicyStore.IsSupportedEdition(edition)) throw new PlatformNotSupportedException($"Unsupported Windows edition: {edition}. No changes made.");
    var engine = new PrivacyEngine(new WindowsPolicyStore());
    var machine = Environment.MachineName;
    if (options.Command == "audit")
    {
        var findings = engine.Audit(policies);
        if (options.Json) Console.WriteLine(JsonSerializer.Serialize(findings, PrivacyBackupFile.Json));
        else
        {
            Console.WriteLine($"Read-only policy registry audit: {edition}. Configured is not proof of effective enforcement. MDM/GPO, per-app overrides and desktop apps need separate review.");
            foreach (var f in findings) Console.WriteLine($"{f.Id}: {f.State}; current={(f.Current?.Exists == true ? f.Current.Value.ToString() : "not configured/unknown")}, desired={f.Desired}\n  {f.Impact}{(f.Error is null ? "" : "\n  Error: " + f.Error)}");
        }
        return findings.Any(f => f.State == "Unknown") ? 2 : 0;
    }
    var backup = options.Command == "restore" ? PrivacyBackupFile.Load(options.Backup!) : engine.Plan(policies, machine);
    PrivacyEngine.Validate(backup, machine);
    Console.WriteLine($"{(options.Apply ? "Requested operation" : "DRY RUN")}: {options.Command}; {backup.Changes.Count} selected changes");
    foreach (var c in backup.Changes)
    {
        var p = PolicyCatalog.Get(c.Id);
        Console.WriteLine($"{c.Id}: before={(c.Before.Exists ? c.Before.Value.ToString() : "absent")}, enforced={c.Desired}\n  {p.Impact}");
    }
    if (options.Command == "restore") engine.Restore(backup, machine, dryRun: true);
    if (!options.Apply || backup.Changes.Count == 0) return 0;
    using var identity = WindowsIdentity.GetCurrent();
    if (!new WindowsPrincipal(identity).IsInRole(WindowsBuiltInRole.Administrator)) throw new UnauthorizedAccessException("Apply/restore requires an elevated terminal. Audit and dry-run do not.");
    if (Console.IsInputRedirected) throw new InvalidOperationException("Interactive confirmation required. Re-run in a terminal; redirected input cannot approve changes.");
    Console.WriteLine("These are machine-wide policies. On organization-managed devices, consult your administrator first. No guarantee of anonymity or zero telemetry. Restart affected apps; some policies may need sign-out/restart.");
    Console.Write($"Type {options.Command.ToUpperInvariant()} to confirm only these policies: ");
    if (Console.ReadLine() != options.Command.ToUpperInvariant()) { Console.WriteLine("Cancelled. No policy writes."); return 0; }
    if (options.Command == "enforce")
    {
        var path = Path.GetFullPath(options.Backup!);
        Console.WriteLine($"Recovery file: {path}. Keep it private and retain it if anything fails.");
        engine.Apply(backup, machine, b => PrivacyBackupFile.Save(path, b));
    }
    else engine.Restore(backup, machine, dryRun: false);
    Console.WriteLine("Registry values verified. Audit effective behavior separately; no services, security controls, tasks or network settings were changed.");
    return 0;
}
catch (Exception ex)
{
    Console.Error.WriteLine($"Stopped: {ex.Message}\nIf an apply/restore began, retain its backup and audit before retrying. Earlier writes may have completed.");
    return 1;
}
