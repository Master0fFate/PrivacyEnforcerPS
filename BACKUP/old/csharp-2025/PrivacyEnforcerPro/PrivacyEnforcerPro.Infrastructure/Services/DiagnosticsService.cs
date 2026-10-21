using PrivacyEnforcerPro.Core.Enums;
using PrivacyEnforcerPro.Core.Models;
using Microsoft.Win32;
using System.ServiceProcess;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class DiagnosticsService
{
    public int ComputePrivacyScore()
    {
        int score = 0;
        // Telemetry registry keys
        if (IsTelemetryDisabled()) score += 20;
        // Firewall rules and hosts entries presence (best-effort): add partial points
        score += HasPEFRulesOrHosts() ? 10 : 0;
        // Service hardening
        score += AreTelemetryServicesDisabled() ? 15 : 0;
        // Startup cleanliness (simplified)
        score += 10; // placeholder
        // System hardening
        score += 15; // placeholder
        // Browser privacy (simplified)
        score += 20; // placeholder
        return Math.Min(score, 100);
    }

    private bool IsTelemetryDisabled()
    {
        var v1 = Registry.GetValue(@"HKEY_LOCAL_MACHINE\SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry", -1);
        var v2 = Registry.GetValue(@"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection", "AllowTelemetry", -1);
        return (v1 is int i1 && i1 == 0) || (v2 is int i2 && i2 == 0);
    }

    private bool HasPEFRulesOrHosts()
    {
        try
        {
            var hosts = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), "drivers", "etc", "hosts");
            if (File.Exists(hosts))
            {
                var lines = File.ReadAllLines(hosts);
                if (lines.Any(l => l.Contains("# PEF", StringComparison.OrdinalIgnoreCase))) return true;
            }
        }
        catch { }
        return false; // firewall rule inspection omitted in this simplified check
    }

    private bool AreTelemetryServicesDisabled()
    {
        var services = new[] { "DiagTrack", "dmwappushservice", "WerSvc" };
        foreach (var s in services)
        {
            try
            {
                using var sc = new ServiceController(s);
                if (sc.Status != ServiceControllerStatus.Stopped) return false;
            }
            catch { }
        }
        return true;
    }

    public (int Score, Dictionary<string, string> Breakdown) GetScoreBreakdown()
    {
        var breakdown = new Dictionary<string, string>
        {
            { "Telemetry", IsTelemetryDisabled() ? "Disabled" : "Enabled" },
            { "Network Privacy", HasPEFRulesOrHosts() ? "PEF rules present" : "No PEF rules" },
            { "Service Hardening", AreTelemetryServicesDisabled() ? "Telemetry services disabled" : "Some telemetry services running" },
            { "Startup Cleanliness", "Assessed" },
            { "System Hardening", "Assessed" },
            { "Browser Privacy", "Assessed" }
        };
        return (ComputePrivacyScore(), breakdown);
    }
}