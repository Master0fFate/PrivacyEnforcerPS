using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;
using System.Diagnostics;

namespace PrivacyEnforcerPro.Modules.SystemRestoreModule;

public sealed class SystemRestoreModule(ProcessService process, ILoggingService logger) : IPrivacyModule
{
    private readonly ProcessService _process = process;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "System Restore";
    public string Description => "Create a system restore point or revert to a previous state.";
    public ModuleCategory Category => ModuleCategory.Hardening; // Reusing existing category

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "Create a system restore point",
        "Launch System Restore utility (rstrui.exe) to revert"
    });

    public Task<ModuleStatus> GetStatusAsync() => Task.FromResult(new ModuleStatus { ModuleName = ModuleName, Status = OperationStatus.Success, Details = "Ready" });

    public async Task<OperationResult> RollbackAsync(string operationId)
    {
        _logger.Information("Launching System Restore utility (rstrui.exe)...");
        var result = await _process.RunAsync("rstrui.exe", "");
        return result.Status == OperationStatus.Success
            ? OperationResult.Success("Launched System Restore.", operationId)
            : OperationResult.Failure($"Failed to launch System Restore: {result.Message}", operationId);
    }

    public async Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");

        _logger.Information("Attempting to create a system restore point...");
        var script = "Enable-ComputerRestore -Drive \"C:\\\"; Checkpoint-Computer -Description 'PrivacyEnforcerPro Point' -RestorePointType 'MODIFY_SETTINGS'";
        var createResult = await _process.RunAsync("powershell.exe", $"-NoProfile -ExecutionPolicy Bypass -Command \"{script}\"");

        if (createResult.Status == OperationStatus.Failure)
        {
            _logger.Warning($"Failed to create restore point: {createResult.Message}. Ensure System Restore is enabled.");
        }
        else
        {
            _logger.Information("System restore point created successfully.");
        }

        _logger.Information("You can revert changes by launching System Restore (rstrui.exe).");
        return OperationResult.Success("System Restore Point creation executed (see log for details). To revert, you can launch rstrui.exe.", opId);
    }
}
