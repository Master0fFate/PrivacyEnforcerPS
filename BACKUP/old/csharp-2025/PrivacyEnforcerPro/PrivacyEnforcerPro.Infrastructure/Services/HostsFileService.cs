using System.Text;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class HostsFileService
{
    private readonly object _lock = new();
    private readonly FileSystemService _fs;
    private const string Tag = "# PEF";

    public HostsFileService(FileSystemService fs) => _fs = fs;

    public string GetHostsPath() =>
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), "drivers", "etc", "hosts");

    public string BackupHosts()
    {
        try
        {
            var hosts = GetHostsPath();
            if (!File.Exists(hosts)) return string.Empty;
            var backupDir = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData),
                "PrivacyEnforcerPro", "Backups");
            return _fs.BackupFile(hosts, backupDir);
        }
        catch
        {
            return string.Empty;
        }
    }

    public void AddTaggedEntries(IEnumerable<string> domains)
    {
        var hostsPath = GetHostsPath();
        lock (_lock)
        {
            try
            {
                if (!File.Exists(hostsPath))
                {
                    File.WriteAllText(hostsPath, string.Empty, Encoding.UTF8);
                }

                var lines = File.ReadAllLines(hostsPath);
                var existing = new HashSet<string>(lines.Select(l => l.Trim()), StringComparer.OrdinalIgnoreCase);
                var toAppend = new List<string>();
                foreach (var d in domains)
                {
                    var entry = $"127.0.0.1 {d} {Tag}";
                    if (!existing.Contains(entry))
                        toAppend.Add(entry);
                }

                if (toAppend.Count > 0)
                {
                    var text = Environment.NewLine + string.Join(Environment.NewLine, toAppend);
                    File.AppendAllText(hostsPath, text, Encoding.UTF8);
                }
            }
            catch
            {
                // Swallow to avoid crashing the app; operation will just be skipped
            }
        }
    }

    public void RemoveTaggedEntries()
    {
        var hostsPath = GetHostsPath();
        lock (_lock)
        {
            try
            {
                if (!File.Exists(hostsPath)) return;
                var filtered = File.ReadLines(hostsPath)
                    .Where(l => !l.Contains(Tag, StringComparison.OrdinalIgnoreCase))
                    .ToArray();
                File.WriteAllLines(hostsPath, filtered, Encoding.UTF8);
            }
            catch
            {
                // Swallow to avoid crashing the app; operation will just be skipped
            }
        }
    }
}