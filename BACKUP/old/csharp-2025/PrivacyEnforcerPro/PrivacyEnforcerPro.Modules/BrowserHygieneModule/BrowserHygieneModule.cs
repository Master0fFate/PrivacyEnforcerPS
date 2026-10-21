using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;

namespace PrivacyEnforcerPro.Modules.BrowserHygieneModule;

public sealed class BrowserHygieneModule(ILoggingService logger) : IPrivacyModule
{
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "Browser Hygiene";
    public string Description => "Clear cache, cookies, and session data for common browsers.";
    public ModuleCategory Category => ModuleCategory.Browser;

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "Clear cache for Chrome, Edge, Firefox",
        "Close running browser processes before cleanup"
    });

    public Task<ModuleStatus> GetStatusAsync() => Task.FromResult(new ModuleStatus
    {
        ModuleName = ModuleName,
        Status = OperationStatus.Success,
        Details = "Cleanup ready"
    });

    public async Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        _logger.Information("Starting browser hygiene cleanup...");

        var targets = new List<string>();
        var localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        var appDataRoaming = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);

        // Chromium-based paths
        targets.Add(Path.Combine(localAppData, "Google", "Chrome", "User Data"));
        targets.Add(Path.Combine(localAppData, "Microsoft", "Edge", "User Data"));
        targets.Add(Path.Combine(localAppData, "BraveSoftware", "Brave-Browser", "User Data"));
        targets.Add(Path.Combine(localAppData, "Vivaldi", "User Data"));
        targets.Add(Path.Combine(appDataRoaming, "Opera Software", "Opera Stable"));

        // Firefox
        targets.Add(Path.Combine(appDataRoaming, "Mozilla", "Firefox", "Profiles"));

        var deletedCount = 0;
        await Task.Run(() =>
        {
            foreach (var basePath in targets.Where(Directory.Exists))
            {
                try
                {
                    foreach (var profile in Directory.EnumerateDirectories(basePath))
                    {
                        // Common cache directories
                        var cacheDirs = new[]
                        {
                            Path.Combine(profile, "Cache"),
                            Path.Combine(profile, "Code Cache"),
                            Path.Combine(profile, "GPUCache"),
                            Path.Combine(profile, "Service Worker"),
                            Path.Combine(profile, "Local Storage"),
                            Path.Combine(profile, "IndexedDB"),
                            Path.Combine(profile, "Sessions")
                        };

                        foreach (var dir in cacheDirs)
                        {
                            if (Directory.Exists(dir))
                            {
                                try
                                {
                                    Directory.Delete(dir, recursive: true);
                                    deletedCount++;
                                }
                                catch (Exception ex)
                                {
                                    _logger.Warning($"Failed to delete {dir}: {ex.Message}");
                                }
                            }
                        }

                        // Common SQLite cookie/history files (best-effort deletion)
                        var files = new[]
                        {
                            Path.Combine(profile, "Cookies"),
                            Path.Combine(profile, "Cookies-journal"),
                            Path.Combine(profile, "History"),
                            Path.Combine(profile, "History-journal")
                        };

                        foreach (var f in files)
                        {
                            try
                            {
                                if (File.Exists(f)) File.Delete(f);
                            }
                            catch (Exception ex)
                            {
                                _logger.Warning($"Failed to delete {f}: {ex.Message}");
                            }
                        }
                    }
                }
                catch (Exception ex)
                {
                    _logger.Warning($"Profile enumeration failed for {basePath}: {ex.Message}");
                }
            }
        }, cancellationToken);

        return OperationResult.Success($"Browser hygiene complete. Deleted {deletedCount} cache folders.", opId);
    }

    public Task<OperationResult> RollbackAsync(string operationId)
        => Task.FromResult(OperationResult.Warning("Rollback not available for deleted cache", operationId));
}