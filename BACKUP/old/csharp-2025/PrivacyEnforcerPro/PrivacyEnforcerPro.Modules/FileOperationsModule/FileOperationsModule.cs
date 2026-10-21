using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;

namespace PrivacyEnforcerPro.Modules.FileOperationsModule;

public sealed class FileOperationsModule(FileOperationsService files, ILoggingService logger) : IPrivacyModule
{
    private readonly FileOperationsService _files = files;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "File Operations";
    public string Description => "Purge Windows log directories.";
    public ModuleCategory Category => ModuleCategory.Files;

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "Purge Windows log directories"
    });

    public Task<ModuleStatus> GetStatusAsync() => Task.FromResult(new ModuleStatus
    {
        ModuleName = ModuleName,
        Status = OperationStatus.Success,
        Details = "Ready"
    });

    public async Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        var logs = new[]
        {
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Windows), "Logs"),
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Windows), "System32", "LogFiles"),
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "Microsoft", "Windows", "WER"),
            Environment.GetEnvironmentVariable("LocalAppData") + "\\Temp"
        };

        _logger.Information("Purging log directories...");
        foreach (var d in logs)
        {
            var exists = Directory.Exists(d);
            _logger.Information($"Target: {d} - {(exists ? "exists" : "missing")}");
        }

        var deleted = 0;
        await Task.Run(() =>
        {
            deleted = _files.PurgeLogDirectories(logs, olderThanDays: 0);
        }, cancellationToken);

        _logger.Information($"Deleted files count: {deleted}");
        return OperationResult.Success($"Purged {deleted} log files.", opId);
    }

    public Task<OperationResult> RollbackAsync(string operationId) => Task.FromResult(OperationResult.Warning("No rollback for deleted files", operationId));
}