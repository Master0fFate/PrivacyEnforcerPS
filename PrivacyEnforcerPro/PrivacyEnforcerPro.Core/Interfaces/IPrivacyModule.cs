namespace PrivacyEnforcerPro.Core.Interfaces;

using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Models;

public interface IPrivacyModule
{
    string ModuleName { get; }
    string Description { get; }
    ModuleCategory Category { get; }

    Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default);
    Task<OperationResult> RollbackAsync(string operationId);
    Task<ModuleStatus> GetStatusAsync();
    Task<List<string>> GetAvailableOperationsAsync();
}