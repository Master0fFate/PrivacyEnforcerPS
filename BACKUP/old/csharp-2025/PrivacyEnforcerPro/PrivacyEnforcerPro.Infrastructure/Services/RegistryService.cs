using Microsoft.Win32;
using PrivacyEnforcerPro.Core.Models;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class RegistryService
{
    public OperationResult SetDword(string keyPath, string valueName, int value)
    {
        try
        {
            Registry.SetValue(keyPath, valueName, value, RegistryValueKind.DWord);
            return OperationResult.Success($"Set {keyPath}\\{valueName} = {value}");
        }
        catch (Exception ex)
        {
            return OperationResult.Failure($"Registry write failed: {ex.Message}");
        }
    }

    public object? GetValue(string keyPath, string valueName, object? defaultValue = null)
    {
        try
        {
            return Registry.GetValue(keyPath, valueName, defaultValue);
        }
        catch
        {
            return defaultValue;
        }
    }
}