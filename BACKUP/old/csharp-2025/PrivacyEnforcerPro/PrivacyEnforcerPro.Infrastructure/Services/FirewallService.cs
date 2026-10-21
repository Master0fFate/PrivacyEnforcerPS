using PrivacyEnforcerPro.Core.Models;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class FirewallService(ProcessService process)
{
    private readonly ProcessService _process = process;

    public Task<OperationResult> AddOutboundBlockRuleAsync(string ruleName, string remoteIp, string protocol = "any")
    {
        var args = $"advfirewall firewall add rule name=\"PEF-{ruleName}\" dir=out action=block remoteip={remoteIp} protocol={protocol} enable=yes profile=any";
        return _process.RunAsync("netsh", args);
    }

    public Task<OperationResult> DeleteRuleAsync(string ruleName)
    {
        var args = $"advfirewall firewall delete rule name=\"PEF-{ruleName}\"";
        return _process.RunAsync("netsh", args);
    }

    public Task<OperationResult> ShowAllRulesAsync()
    {
        return _process.RunAsync("netsh", "advfirewall firewall show rule name=all");
    }
}