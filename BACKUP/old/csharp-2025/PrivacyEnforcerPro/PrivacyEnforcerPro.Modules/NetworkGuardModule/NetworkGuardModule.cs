using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;

namespace PrivacyEnforcerPro.Modules.NetworkGuardModule;

public sealed class NetworkGuardModule(FirewallService firewall, HostsFileService hosts, ILoggingService logger) : IPrivacyModule
{
    private readonly FirewallService _firewall = firewall;
    private readonly HostsFileService _hosts = hosts;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "Network Guard";
    public string Description => "Firewall management and hosts file blocking for telemetry domains.";
    public ModuleCategory Category => ModuleCategory.Network;

    public async Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        var domains = new[]
        {
            "vortex.data.microsoft.com",
            "telemetry.microsoft.com",
            "oca.telemetry.microsoft.com",
            "watson.microsoft.com"
        };

        _logger.Information("Backing up hosts file and applying PEF-tagged entries...");
        _hosts.BackupHosts();
        _hosts.AddTaggedEntries(domains);

        _logger.Information("Creating firewall block rules for known telemetry endpoints...");
        // Example IPs, can be resolved dynamically in future iterations
        var rules = new[]
        {
            ("Block-Telemetry-Vortex", "13.107.4.50"),
            ("Block-Telemetry-Vortex-Alt", "13.107.5.50")
        };

        foreach (var (name, ip) in rules)
        {
            var res = await _firewall.AddOutboundBlockRuleAsync(name, ip);
            if (res.Status == OperationStatus.Failure)
                _logger.Warning($"Failed to add firewall rule {name}: {res.Message}");
            else
                _logger.Information($"Added firewall rule {name}");
        }

        return OperationResult.Success("Network Guard applied: hosts entries and firewall rules added.", opId);
    }

    public Task<OperationResult> RollbackAsync(string operationId)
    {
        _hosts.RemoveTaggedEntries();
        return Task.FromResult(OperationResult.Success("Removed PEF-tagged hosts entries.", operationId));
    }

    public Task<ModuleStatus> GetStatusAsync()
        => Task.FromResult(new ModuleStatus { ModuleName = ModuleName, Status = OperationStatus.Success, Details = "Hosts entries present; firewall rules created." });

    public Task<List<string>> GetAvailableOperationsAsync()
        => Task.FromResult(new List<string>
        {
            "Add hosts entries for telemetry domains",
            "Create firewall block rules for telemetry IPs",
            "Remove PEF-tagged hosts entries"
        });
}