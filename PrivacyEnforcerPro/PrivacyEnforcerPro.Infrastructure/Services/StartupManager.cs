using Microsoft.Win32;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class StartupManager
{
    public record StartupItem(string Name, string Path, string Location, bool Enabled, string Publisher = "", string Impact = "Unknown");

    private static readonly (RegistryKey Key, string Location)[] RunKeys = new[]
    {
        (Registry.CurrentUser.OpenSubKey("Software\\Microsoft\\Windows\\CurrentVersion\\Run", writable: true)!, "HKCU Run"),
        (Registry.LocalMachine.OpenSubKey("Software\\Microsoft\\Windows\\CurrentVersion\\Run", writable: true)!, "HKLM Run"),
        (Registry.CurrentUser.OpenSubKey("Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce", writable: true)!, "HKCU RunOnce"),
        (Registry.LocalMachine.OpenSubKey("Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce", writable: true)!, "HKLM RunOnce"),
    };

    public IEnumerable<StartupItem> Enumerate()
    {
        foreach (var (key, location) in RunKeys)
        {
            if (key is null) continue;
            foreach (var name in key.GetValueNames())
            {
                var path = key.GetValue(name)?.ToString() ?? string.Empty;
                yield return new StartupItem(name, path, location, true);
            }
        }

        var userStartup = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Microsoft", "Windows", "Start Menu", "Programs", "Startup");
        var commonStartup = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "Microsoft", "Windows", "Start Menu", "Programs", "Startup");

        foreach (var file in Directory.Exists(userStartup) ? Directory.EnumerateFiles(userStartup) : Array.Empty<string>())
            yield return new StartupItem(Path.GetFileName(file), file, "User Startup Folder", true);
        foreach (var file in Directory.Exists(commonStartup) ? Directory.EnumerateFiles(commonStartup) : Array.Empty<string>())
            yield return new StartupItem(Path.GetFileName(file), file, "Common Startup Folder", true);
    }

    public void DisableRegistryStartup(string location, string name)
    {
        var (key, _) = RunKeys.FirstOrDefault(r => r.Location == location);
        key?.DeleteValue(name, throwOnMissingValue: false);
    }

    public void RemoveStartupFile(string path)
    {
        if (File.Exists(path)) File.Delete(path);
    }
}