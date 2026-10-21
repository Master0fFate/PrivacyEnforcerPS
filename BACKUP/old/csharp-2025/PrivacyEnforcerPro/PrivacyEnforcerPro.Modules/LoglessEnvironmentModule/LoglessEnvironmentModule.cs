using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;
using System.Diagnostics;

namespace PrivacyEnforcerPro.Modules.LoglessEnvironmentModule;

public sealed class LoglessEnvironmentModule(RegistryService registry, ProcessService process, ILoggingService logger) : IPrivacyModule
{
    private readonly RegistryService _registry = registry;
    private readonly ProcessService _process = process;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "Log-less Environment";
    public string Description => "Comprehensive zero-log environment: Disable user tracking, wipe logs, delete journal, and clear MRUs.";
    public ModuleCategory Category => ModuleCategory.Hardening;

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "Disable Activity Feed (Timeline) & user activity tracking",
        "Disable UserAssist (track launched programs)",
        "Disable Recent Documents & Quick Access history",
        "Disable Prefetch and Superfetch (usage tracking)",
        "Disable PowerShell Script Block Logging",
        "Disable Windows Error Reporting (WER)",
        "Disable Clipboard History",
        "Clear All Windows Event Logs (wevtutil)",
        "Delete NTFS USN Journal (fsutil)",
        "Clear Registry MRU Caches (RunMRU, TypedPaths, WordWheel)",
        "Empty Prefetch and Temp folders"
    });

    public Task<ModuleStatus> GetStatusAsync() => Task.FromResult(new ModuleStatus { ModuleName = ModuleName, Status = OperationStatus.Success, Details = "Ready" });

    public Task<OperationResult> RollbackAsync(string operationId) => Task.FromResult(OperationResult.Warning("Rollback via System Restore is recommended.", operationId));

    public async Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        _logger.Information("Applying comprehensive Log-less Environment tweaks...");
        var results = new List<OperationResult>();

        // Activity Feed / Timeline / User Activity
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\System", "EnableActivityFeed", 0));
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\System", "PublishUserActivities", 0));
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\System", "UploadUserActivities", 0));

        // UserAssist
        results.Add(_registry.SetDword(@"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "Start_TrackProgs", 0));

        // Recent Documents / Quick Access
        results.Add(_registry.SetDword(@"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer", "NoRecentDocsHistory", 1));
        results.Add(_registry.SetDword(@"HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer", "ClearRecentDocsOnExit", 1));
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer", "NoRecentDocsHistory", 1));

        // Disable Prefetch / Superfetch (SysMain) tracking
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters", "EnablePrefetcher", 0));
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters", "EnableSuperfetch", 0));

        // Disable PowerShell Logging (Script Block and Transcription)
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging", "EnableScriptBlockLogging", 0));
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\PowerShell\Transcription", "EnableTranscripting", 0));

        // Disable Windows Error Reporting (WER)
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting", "Disabled", 1));

        // Disable Clipboard History
        results.Add(_registry.SetDword(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\System", "AllowClipboardHistory", 0));

        // Registry MRU Cleanup via PowerShell Script
        var script = @"
            $ErrorActionPreference = 'SilentlyContinue'
            # Clear Run MRU
            Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU' -Name '*' -Force
            # Clear TypedPaths
            Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths' -Name '*' -Force
            # Clear WordWheelQuery (Search History)
            Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery' -Name '*' -Force
            # Clear MuiCache
            Remove-Item -Path 'HKCU:\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache' -Recurse -Force
            # Delete USN Journal
            fsutil usn deletejournal /d c:
            # Clear Windows Event Logs
            Get-WinEvent -ListLog * | Where-Object {$_.RecordCount -gt 0} | ForEach-Object { wevtutil cl $_.LogName }
            # Clear Prefetch
            Remove-Item -Path 'C:\Windows\Prefetch\*' -Recurse -Force
            # Clear Temp
            Remove-Item -Path 'C:\Windows\Temp\*' -Recurse -Force
            Remove-Item -Path $env:TEMP\* -Recurse -Force
        ";

        _logger.Information("Executing log destruction script (Event Logs, Journal, MRUs, Temp, Prefetch)...");
        var psResult = await _process.RunAsync("powershell.exe", $"-NoProfile -ExecutionPolicy Bypass -Command \"{script}\"");

        if (psResult.Status == OperationStatus.Failure)
            _logger.Warning($"Some log destruction tasks encountered errors: {psResult.Message}");
        else
            _logger.Information("Log destruction script executed successfully.");

        var failures = results.Count(r => r.Status == OperationStatus.Failure);
        var msg = failures == 0 ? "Log-less profile and log destruction applied successfully." : $"Applied log-less profile with {failures} registry tweaks failures. See logs.";

        return failures == 0 && psResult.Status == OperationStatus.Success
            ? OperationResult.Success(msg, opId)
            : OperationResult.Warning(msg, opId);
    }
}
