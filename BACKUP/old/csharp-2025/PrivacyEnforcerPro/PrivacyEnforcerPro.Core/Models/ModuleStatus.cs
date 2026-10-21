using PrivacyEnforcerPro.Core.Enums;

namespace PrivacyEnforcerPro.Core.Models;

public sealed class ModuleStatus
{
    public string ModuleName { get; init; } = string.Empty;
    public OperationStatus Status { get; init; } = OperationStatus.Success;
    public string Details { get; init; } = string.Empty;
}