using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;
using Spectre.Console;

namespace PrivacyEnforcerPro.Modules.DiagnosticsModule;

public sealed class DiagnosticsModule(DiagnosticsService diagnostics, ILoggingService logger) : IPrivacyModule
{
    private readonly DiagnosticsService _diagnostics = diagnostics;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "Diagnostics & Reports";
    public string Description => "Assess privacy posture and show a report.";
    public ModuleCategory Category => ModuleCategory.Diagnostics;

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "Compute privacy score and display breakdown"
    });

    public Task<ModuleStatus> GetStatusAsync() => Task.FromResult(new ModuleStatus { ModuleName = ModuleName, Status = OperationStatus.Success, Details = "Ready" });

    public Task<OperationResult> RollbackAsync(string operationId) => Task.FromResult(OperationResult.Success("Diagnostics are read-only", operationId));

    public Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        var (score, breakdown) = _diagnostics.GetScoreBreakdown();
        var table = new Table().AddColumn("Category").AddColumn("Status");
        foreach (var kv in breakdown) table.AddRow(kv.Key, kv.Value);
        table.AddRow("Privacy Score", score.ToString());
        AnsiConsole.Write(table);
        return Task.FromResult(OperationResult.Success("Diagnostics computed and displayed.", opId));
    }
}