using Microsoft.Win32;
using PrivacyEnforcerPro.Core.Models;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class HardeningService
{
    private OperationResult SetDword(string keyPath, string valueName, int value)
    {
        try
        {
            Registry.SetValue(keyPath, valueName, value, RegistryValueKind.DWord);
            return OperationResult.Success($"Set {keyPath}\\{valueName}={value}");
        }
        catch (Exception ex)
        {
            return OperationResult.Failure(ex.Message);
        }
    }

    public IEnumerable<OperationResult> ApplyBasicHardening()
    {
        var results = new List<OperationResult>();
        // Disable LLMNR
        results.Add(SetDword(@"HKEY_LOCAL_MACHINE\Software\Policies\Microsoft\Windows NT\DNSClient", "EnableMulticast", 0));
        // Disable WPAD - WinHttpAutoProxySvc via services profile or policy is complex; here disable AutoProxy result cache
        results.Add(SetDword(@"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Internet Settings", "EnableAutoProxyResultCache", 0));
        // Disable Autoplay
        results.Add(SetDword(@"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer", "NoDriveTypeAutoRun", 255));
        // UAC to highest consent
        results.Add(SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System", "ConsentPromptBehaviorAdmin", 2));
        results.Add(SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System", "EnableLUA", 1));
        // SmartScreen enable
        results.Add(SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer", "SmartScreenEnabled", 1));
        // Disable Remote Assistance
        results.Add(SetDword(@"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Remote Assistance", "fAllowToGetHelp", 0));
        // Disable Camera access for apps (capability toggle placeholder)
        results.Add(SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\webcam", "Value", 0));
        // Disable Microphone access for apps
        results.Add(SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone", "Value", 0));

        return results;
    }
}