using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;
using System.ServiceProcess;

namespace PrivacyEnforcerPro.Modules.TelemetryModule;

public sealed class TelemetryEliminationModule(RegistryService registry, ProcessService process, ILoggingService logger) : IPrivacyModule
{
    private readonly RegistryService _registry = registry;
    private readonly ProcessService _process = process;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "Telemetry Elimination";
    public string Description => "Disable Windows telemetry, diagnostics, and related scheduled tasks/services.";
    public ModuleCategory Category => ModuleCategory.Telemetry;

    public async Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        _logger.Information("Starting telemetry elimination operations...");

        // Registry modifications
        var regOps = new List<OperationResult>
        {
            _registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry", 0),
            _registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection", "AllowTelemetry", 0),
            _registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\CloudContent", "DisableWindowsConsumerFeatures", 1),
            _registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo", "DisabledByGroupPolicy", 1),
            _registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\System", "EnableActivityFeed", 0)
        };

        // Services to disable
        var services = new[] { "DiagTrack", "dmwappushservice", "WerSvc" };
        foreach (var svcName in services)
        {
            try
            {
                using var sc = new ServiceController(svcName);
                if (sc.Status != ServiceControllerStatus.Stopped)
                {
                    sc.Stop();
                    sc.WaitForStatus(ServiceControllerStatus.Stopped, TimeSpan.FromSeconds(10));
                }
                // Set startup type to Disabled via sc.exe
                var setStartup = await _process.RunAsync("sc", $"config {svcName} start= disabled");
                if (setStartup.Status == OperationStatus.Failure)
                    _logger.Warning($"Failed to set startup type for {svcName}: {setStartup.Message}");
                else
                    _logger.Information($"Disabled service {svcName}");
            }
            catch (Exception ex)
            {
                _logger.Error($"Service operation failed for {svcName}", ex);
            }
        }

        // Scheduled tasks to disable
        var tasks = new[]
        {
            "\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser",
            "\\Microsoft\\Windows\\Application Experience\\ProgramDataUpdater",
            "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator",
            "\\Microsoft\\Windows\\Autochk\\Proxy",
            "\\Microsoft\\Windows\\CloudExperienceHost\\FirstLogonAnimation"
        };

        foreach (var task in tasks)
        {
            var res = await _process.RunAsync("schtasks", $"/Change /TN \"{task}\" /DISABLE");
            if (res.Status == OperationStatus.Failure)
                _logger.Warning($"Failed to disable task {task}: {res.Message}");
            else
                _logger.Information($"Disabled task {task}");
        }

        var failures = regOps.Count(r => r.Status == OperationStatus.Failure);
        var message = failures == 0 ? "Telemetry elimination completed." : $"Telemetry elimination completed with {failures} registry write failures.";
        return failures == 0 ? OperationResult.Success(message, opId) : OperationResult.Warning(message, opId);
    }

    public Task<OperationResult> RollbackAsync(string operationId)
        => Task.FromResult(OperationResult.Warning("Rollback not yet implemented", operationId));

    public Task<ModuleStatus> GetStatusAsync()
        => Task.FromResult(new ModuleStatus { ModuleName = ModuleName, Status = OperationStatus.Success, Details = "Basic telemetry settings applied." });

    public Task<List<string>> GetAvailableOperationsAsync()
        => Task.FromResult(new List<string>
        {
            "Disable telemetry registry keys",
            "Disable telemetry services",
            "Disable telemetry scheduled tasks"
        });
}