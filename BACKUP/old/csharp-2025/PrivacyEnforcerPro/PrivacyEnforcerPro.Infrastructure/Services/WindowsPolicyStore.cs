using Microsoft.Win32;
using PrivacyEnforcerPro.Core.Policies;
using System.Runtime.Versioning;

namespace PrivacyEnforcerPro.Infrastructure.Services;

[SupportedOSPlatform("windows")]
public sealed class WindowsPolicyStore : IPolicyStore
{
    private static void Validate(PrivacyPolicy policy)
    {
        if (PolicyCatalog.Get(policy.Id) != policy) throw new ArgumentException("Policy is outside the built-in catalog.");
    }
    public PolicyValue Read(PrivacyPolicy policy)
    {
        Validate(policy);
        using var hive = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);
        using var key = hive.OpenSubKey(policy.Key, writable: false);
        if (key is null || !key.GetValueNames().Contains(policy.Name, StringComparer.OrdinalIgnoreCase)) return new(false);
        if (key.GetValueKind(policy.Name) != RegistryValueKind.DWord) throw new InvalidOperationException($"{policy.Id} has an unexpected registry type; refusing conversion.");
        return new(true, (int)key.GetValue(policy.Name)!);
    }
    public void Write(PrivacyPolicy policy, PolicyValue value)
    {
        Validate(policy);
        using var hive = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);
        if (value.Exists)
        {
            using var key = hive.CreateSubKey(policy.Key, writable: true);
            key.SetValue(policy.Name, value.Value, RegistryValueKind.DWord);
            key.Flush();
        }
        else
        {
            using var key = hive.OpenSubKey(policy.Key, writable: true);
            key?.DeleteValue(policy.Name, throwOnMissingValue: false);
            key?.Flush(); // Do not remove shared keys or other values.
        }
    }

    public static string Edition()
    {
        using var hive = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);
        using var key = hive.OpenSubKey(@"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
        return key?.GetValue("EditionID") as string ?? "Unknown";
    }
    public static bool IsSupportedEdition(string edition) => edition is
        "Professional" or "ProfessionalN" or "ProfessionalEducation" or "ProfessionalWorkstation" or
        "Enterprise" or "EnterpriseN" or "EnterpriseS" or "Education" or "EducationN";
}
