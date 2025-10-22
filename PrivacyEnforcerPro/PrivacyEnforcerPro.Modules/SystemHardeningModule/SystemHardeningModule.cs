using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;

namespace PrivacyEnforcerPro.Modules.SystemHardeningModule;

public sealed class SystemHardeningModule(HardeningService hardening, ILoggingService logger) : IPrivacyModule
{
    private readonly HardeningService _hardening = hardening;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "System Hardening";
    public string Description => "Apply security best practices and disable vulnerable features.";
    public ModuleCategory Category => ModuleCategory.Hardening;

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "Apply basic hardening profile (LLMNR, Autoplay, UAC, SmartScreen)"
    });

    public Task<ModuleStatus> GetStatusAsync() => Task.FromResult(new ModuleStatus { ModuleName = ModuleName, Status = OperationStatus.Success, Details = "Ready" });

    public Task<OperationResult> RollbackAsync(string operationId) => Task.FromResult(OperationResult.Warning("Rollback not implemented for hardening", operationId));

    public Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        var results = _hardening.ApplyBasicHardening().ToList();
        var failures = results.Count(r => r.Status == OperationStatus.Failure);
        var msg = failures == 0 ? "Applied hardening profile." : $"Applied with {failures} failures.";
        return Task.FromResult(failures == 0 ? OperationResult.Success(msg, opId) : OperationResult.Warning(msg, opId));
    }
}