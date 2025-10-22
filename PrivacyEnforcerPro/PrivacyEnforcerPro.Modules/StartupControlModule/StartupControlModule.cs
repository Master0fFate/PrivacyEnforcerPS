using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;
using Spectre.Console;

namespace PrivacyEnforcerPro.Modules.StartupControlModule;

public sealed class StartupControlModule(StartupManager startup, ILoggingService logger) : IPrivacyModule
{
    private readonly StartupManager _startup = startup;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "Startup Control";
    public string Description => "Manage startup items from registry and startup folders.";
    public ModuleCategory Category => ModuleCategory.Startup;

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "List startup items",
        "Disable registry Run entries",
        "Remove startup folder shortcuts"
    });

    public Task<ModuleStatus> GetStatusAsync() => Task.FromResult(new ModuleStatus { ModuleName = ModuleName, Status = OperationStatus.Success, Details = "Ready" });

    public Task<OperationResult> RollbackAsync(string operationId) => Task.FromResult(OperationResult.Warning("No rollback for deleted startup items", operationId));

    public Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        var items = _startup.Enumerate().ToList();
        var table = new Table().AddColumn("Name").AddColumn("Location").AddColumn("Path");
        foreach (var i in items.Take(20)) table.AddRow(i.Name, i.Location, i.Path);
        AnsiConsole.Write(table);

        foreach (var i in items.Where(i => i.Location.Contains("Run")))
        {
            // Example: disable high-impact known offenders; in full implementation, prompt user selection
            if (i.Name.Contains("OneDrive", StringComparison.OrdinalIgnoreCase) || i.Name.Contains("Teams", StringComparison.OrdinalIgnoreCase))
            {
                _startup.DisableRegistryStartup(i.Location, i.Name);
                _logger.Information($"Disabled startup registry entry: {i.Name}");
            }
        }

        return Task.FromResult(OperationResult.Success("Listed startup items and disabled known offenders.", opId));
    }
}