namespace PrivacyEnforcerPro.Core.Policies;

public sealed record CommandOptions(string Command, string[] Ids, string? Backup, bool Apply, bool Json)
{
    public static CommandOptions Parse(string[] args)
    {
        var command = args.Length == 0 ? "audit" : args[0];
        if (command is not ("audit" or "list" or "enforce" or "restore" or "help")) throw new ArgumentException("Unknown command. Run help.");
        var options = new Dictionary<string, string?>(StringComparer.Ordinal);
        for (var i = 1; i < args.Length; i++)
        {
            var key = args[i];
            if (key is not ("--select" or "--backup" or "--apply" or "--json") || options.ContainsKey(key)) throw new ArgumentException($"Unknown or repeated option: {key}");
            string? value = null;
            if (key is "--select" or "--backup")
            {
                if (++i >= args.Length || args[i].StartsWith("--") || string.IsNullOrWhiteSpace(args[i])) throw new ArgumentException($"Missing value for {key}");
                value = args[i];
            }
            options.Add(key, value);
        }
        if ((command is "audit" or "list" or "help") && (options.ContainsKey("--apply") || options.ContainsKey("--backup"))) throw new ArgumentException("Read-only commands do not accept --apply or --backup.");
        if (command == "restore" && options.ContainsKey("--select")) throw new ArgumentException("Restore uses the backup's policy selection.");
        if (command is "enforce" or "restore" && options.ContainsKey("--json")) throw new ArgumentException("--json is only supported for read-only output.");
        var ids = options.GetValueOrDefault("--select")?.Split(',', StringSplitOptions.TrimEntries) ?? [];
        if (command == "enforce" || ids.Length > 0) PolicyCatalog.Select(ids);
        var backup = options.GetValueOrDefault("--backup");
        if ((command == "restore" || command == "enforce" && options.ContainsKey("--apply")) && backup is null) throw new ArgumentException("This operation requires --backup PATH.");
        return new(command, ids, backup, options.ContainsKey("--apply"), options.ContainsKey("--json"));
    }
}
