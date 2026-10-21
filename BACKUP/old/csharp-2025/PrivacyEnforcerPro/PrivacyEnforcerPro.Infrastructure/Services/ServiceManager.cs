using System.ServiceProcess;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Core.Enums;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class ServiceManager(ProcessService process)
{
    private readonly ProcessService _process = process;

    public IEnumerable<(string Name, string DisplayName, ServiceControllerStatus Status, ServiceStartMode StartType)> Enumerate()
    {
        foreach (var s in ServiceController.GetServices())
        {
            ServiceStartMode startMode = ServiceStartMode.Manual;
            try
            {
                // `sc qc` to query start mode
                var output = _process.RunAsync("sc", $"qc {s.ServiceName}").Result.Message;
                if (output.Contains("AUTO_START", StringComparison.OrdinalIgnoreCase)) startMode = ServiceStartMode.Automatic;
                else if (output.Contains("DEMAND_START", StringComparison.OrdinalIgnoreCase)) startMode = ServiceStartMode.Manual;
                else if (output.Contains("DISABLED", StringComparison.OrdinalIgnoreCase)) startMode = ServiceStartMode.Disabled;
            }
            catch { }
            yield return (s.ServiceName, s.DisplayName, s.Status, startMode);
        }
    }

    public async Task<OperationResult> SetStartupTypeAsync(string serviceName, ServiceStartMode mode)
    {
        var modeStr = mode switch
        {
            ServiceStartMode.Automatic => "auto",
            ServiceStartMode.Manual => "demand",
            ServiceStartMode.Disabled => "disabled",
            _ => "demand"
        };
        return await _process.RunAsync("sc", $"config {serviceName} start= {modeStr}");
    }

    public async Task<OperationResult> StopServiceAsync(string serviceName)
    {
        try
        {
            using var sc = new ServiceController(serviceName);
            if (sc.Status != ServiceControllerStatus.Stopped)
            {
                sc.Stop();
                sc.WaitForStatus(ServiceControllerStatus.Stopped, TimeSpan.FromSeconds(10));
            }
            return OperationResult.Success($"Stopped {serviceName}");
        }
        catch (Exception ex)
        {
            return OperationResult.Failure(ex.Message);
        }
    }
}