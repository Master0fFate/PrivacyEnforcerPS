using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Interfaces;
using PrivacyEnforcerPro.Core.Models;
using PrivacyEnforcerPro.Infrastructure.Services;
using System.Diagnostics;

namespace PrivacyEnforcerPro.Modules.LoglessEnvironmentModule;

public sealed class LoglessEnvironmentModule(RegistryService registry, ILoggingService logger) : IPrivacyModule
{
    private readonly RegistryService _registry = registry;
    private readonly ILoggingService _logger = logger;

    public string ModuleName => "Log-less Environment";
    public string Description => "Stop Windows from logging user actions, recent documents, activity history, and tracking usage.";
    public ModuleCategory Category => ModuleCategory.Hardening;

    public Task<List<string>> GetAvailableOperationsAsync() => Task.FromResult(new List<string>
    {
        "Disable Activity Feed (Timeline) & user activity tracking",
        "Disable UserAssist (track launched programs)",
        "Disable Recent Documents & Quick Access history",
        "Disable Prefetch and Superfetch (usage tracking)",
        "Disable PowerShell Script Block Logging",
        "Disable Application Compatibility Cache (Shimcache)"
    });

    public Task<ModuleStatus> GetStatusAsync() => Task.FromResult(new ModuleStatus { ModuleName = ModuleName, Status = OperationStatus.Success, Details = "Ready" });

    public Task<OperationResult> RollbackAsync(string operationId) => Task.FromResult(OperationResult.Warning("Rollback via System Restore is recommended.", operationId));

    public Task<OperationResult> ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var opId = Guid.NewGuid().ToString("N");
        _logger.Information("Applying Log-less Environment tweaks...");
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

        var failures = results.Count(r => r.Status == OperationStatus.Failure);
        var msg = failures == 0 ? "Log-less profile applied successfully." : $"Applied log-less profile with {failures} failures.";

        return Task.FromResult(failures == 0 ? OperationResult.Success(msg, opId) : OperationResult.Warning(msg, opId));
    }
}
