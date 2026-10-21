using PrivacyEnforcerPro.Core.Enums;

namespace PrivacyEnforcerPro.Core.Models;

public sealed class ModuleMetadata
{
    public string Name { get; init; } = string.Empty;
    public string Description { get; init; } = string.Empty;
    public ModuleCategory Category { get; init; }
}