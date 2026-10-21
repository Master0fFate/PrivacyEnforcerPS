using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;

namespace PrivacyEnforcerPro.Modules.ServiceManagementModule;

public sealed class ServiceManagementModule(ServiceManager svc, ILoggingService logger) : IPrivacyModule
{
    private readonly ServiceManager _svc = svc;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "Service Management";
    public string Description => "Disable unnecessary services based on preset privacy profile.";
    public ModuleCategory Category => ModuleCategory.Services;

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "Enumerate services and display status",
        "Apply Privacy profile to disable telemetry and Xbox services"
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
        var profileDisable = new[] { "DiagTrack", "dmwappushservice", "WerSvc", "XblAuthManager", "XblGameSave", "XboxNetApiSvc" };

        foreach (var svcName in profileDisable)
        {
            var stop = await _svc.StopServiceAsync(svcName);
            if (stop.Status == OperationStatus.Failure) _logger.Warning($"Stop failed for {svcName}: {stop.Message}");
            var disable = await _svc.SetStartupTypeAsync(svcName, System.ServiceProcess.ServiceStartMode.Disabled);
            if (disable.Status == OperationStatus.Failure) _logger.Warning($"Disable failed for {svcName}: {disable.Message}");
        }

        return OperationResult.Success("Applied Service Management privacy profile.", opId);
    }

    public Task<OperationResult> RollbackAsync(string operationId)
        => Task.FromResult(OperationResult.Warning("Rollback not implemented for service profiles", operationId));
}