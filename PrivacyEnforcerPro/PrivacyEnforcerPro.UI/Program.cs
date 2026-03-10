using Spectre.Console;
using Serilog;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using PrivacyEnforcerPro.Infrastructure.Security;
using PrivacyEnforcerPro.Infrastructure.Services;
using PrivacyEnforcerPro.Modules.TelemetryModule;
using PrivacyEnforcerPro.Modules.NetworkGuardModule;
using PrivacyEnforcerPro.Modules.BrowserHygieneModule;
using PrivacyEnforcerPro.Modules.FileOperationsModule;
using PrivacyEnforcerPro.Modules.ServiceManagementModule;
using PrivacyEnforcerPro.Modules.StartupControlModule;
using PrivacyEnforcerPro.Modules.SystemHardeningModule;
using PrivacyEnforcerPro.Modules.DiagnosticsModule;
using PrivacyEnforcerPro.Modules.SystemRestoreModule;
using PrivacyEnforcerPro.Modules.LoglessEnvironmentModule;
using PrivacyEnforcerPro.Core.Interfaces;
using System.Diagnostics;
using System.Text;

var programData = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "PrivacyEnforcerPro");
var logsDir = Path.Combine(programData, "Logs");
Directory.CreateDirectory(logsDir);

// Ensure emoji and Unicode render correctly
Console.OutputEncoding = Encoding.UTF8;
Console.InputEncoding = Encoding.UTF8;

var logFile = Path.Combine(logsDir, $"operations_{DateTime.UtcNow:yyyy-MM-dd}.log");
Log.Logger = new LoggerConfiguration()
    .MinimumLevel.Information()
    .WriteTo.Console()
    .WriteTo.File(logFile)
    .CreateLogger();

if (!PrivilegeElevation.IsAdministrator())
{
    AnsiConsole.MarkupLine("[red]ERROR:[/] This application requires administrator privileges.");
    AnsiConsole.MarkupLine("Please right-click and select 'Run as administrator'.");
    return;
}

var osVersion = Environment.OSVersion;
if (osVersion.Version.Major < 10 || (osVersion.Version.Major == 10 && osVersion.Version.Build < 22000))
{
    AnsiConsole.MarkupLine("[yellow]WARNING:[/] This application is optimized for Windows 11.");
    AnsiConsole.MarkupLine("Some features may not work correctly on older versions.");
}

var builder = Host.CreateDefaultBuilder(args)
    .UseSerilog()
    .ConfigureServices(services =>
    {
        services.AddSingleton<ILoggingService, LoggingService>();
        services.AddSingleton<RegistryService>();
        services.AddSingleton<ProcessService>();
        services.AddSingleton<FileSystemService>();
        services.AddSingleton<EncryptedConfigurationService>();
        services.AddSingleton<FirewallService>();
        services.AddSingleton<HostsFileService>();
        services.AddSingleton<FileOperationsService>();
        services.AddSingleton<ServiceManager>();
        services.AddSingleton<StartupManager>();
        services.AddSingleton<HardeningService>();
        services.AddSingleton<DiagnosticsService>();

        services.AddSingleton<IPrivacyModule, TelemetryEliminationModule>();
        services.AddSingleton<IPrivacyModule, NetworkGuardModule>();
        services.AddSingleton<IPrivacyModule, BrowserHygieneModule>();
        services.AddSingleton<IPrivacyModule, FileOperationsModule>();
        services.AddSingleton<IPrivacyModule, ServiceManagementModule>();
        services.AddSingleton<IPrivacyModule, StartupControlModule>();
        services.AddSingleton<IPrivacyModule, SystemHardeningModule>();
        services.AddSingleton<IPrivacyModule, DiagnosticsModule>();
        services.AddSingleton<IPrivacyModule, SystemRestoreModule>();
        services.AddSingleton<IPrivacyModule, LoglessEnvironmentModule>();
    });

using var host = builder.Build();
var serviceProvider = host.Services;

await ShowMainMenuAsync(serviceProvider, logFile);

static async Task ShowMainMenuAsync(IServiceProvider sp, string logFile)
{
    while (true)
    {
        var panel = new Panel(new Markup("[bold yellow]PRIVACY ENFORCER PRO v2.0 - Windows 11 Edition[/]\n[cyan]Administrator Mode[/]"))
        {
            Border = BoxBorder.Double,
            Expand = true
        };
        AnsiConsole.Write(panel);

        // Plain text lines to avoid markup parsing issues with brackets
        AnsiConsole.WriteLine("\n(1) 🛡️ Telemetry Elimination");
        AnsiConsole.WriteLine("(2) 🌐 Network Guard");
        AnsiConsole.WriteLine("(3) 🔒 Browser Hygiene");
        AnsiConsole.WriteLine("(4) 📁 File Operations");
        AnsiConsole.WriteLine("(5) ⚙️ Service Management");
        AnsiConsole.WriteLine("(6) 🚀 Startup Control");
        AnsiConsole.WriteLine("(7) 🔐 System Hardening");
        AnsiConsole.WriteLine("(8) 📊 Diagnostics & Reports");
        AnsiConsole.WriteLine("(9) ⏪ System Restore Point");
        AnsiConsole.WriteLine("(10) 🕵️ Log-less Environment");
        AnsiConsole.WriteLine("\n(0) ❌ Exit\n");

        var choice = AnsiConsole.Ask<string>("Select module (1-10, 0 to exit):");
        IPrivacyModule? module = choice switch
        {
            "1" => sp.GetServices<IPrivacyModule>().OfType<TelemetryEliminationModule>().FirstOrDefault(),
            "2" => sp.GetServices<IPrivacyModule>().OfType<NetworkGuardModule>().FirstOrDefault(),
            "3" => sp.GetServices<IPrivacyModule>().OfType<BrowserHygieneModule>().FirstOrDefault(),
            "4" => sp.GetServices<IPrivacyModule>().OfType<FileOperationsModule>().FirstOrDefault(),
            "5" => sp.GetServices<IPrivacyModule>().OfType<ServiceManagementModule>().FirstOrDefault(),
            "6" => sp.GetServices<IPrivacyModule>().OfType<StartupControlModule>().FirstOrDefault(),
            "7" => sp.GetServices<IPrivacyModule>().OfType<SystemHardeningModule>().FirstOrDefault(),
            "8" => sp.GetServices<IPrivacyModule>().OfType<DiagnosticsModule>().FirstOrDefault(),
            "9" => sp.GetServices<IPrivacyModule>().OfType<SystemRestoreModule>().FirstOrDefault(),
            "10" => sp.GetServices<IPrivacyModule>().OfType<LoglessEnvironmentModule>().FirstOrDefault(),
            "0" => null,
            _ => null
        };

        if (choice == "0") return;
        if (module is null)
        {
            AnsiConsole.MarkupLine("[red]Module not available.[/]");
            continue;
        }

        // Verbose header
        AnsiConsole.MarkupLine($"[bold cyan]▶ {Markup.Escape(module.ModuleName)}[/] — {Markup.Escape(module.Description)}");
        var availableOps = await module.GetAvailableOperationsAsync();
        if (availableOps.Count > 0)
        {
            AnsiConsole.MarkupLine("[dim]Planned operations:[/]");
            foreach (var op in availableOps)
            {
                AnsiConsole.MarkupLine($"  • {Markup.Escape(op)}");
            }
        }

        PrivacyEnforcerPro.Core.Models.OperationResult result;
        var sw = Stopwatch.StartNew();
        try
        {
            result = await AnsiConsole.Status().StartAsync($"Running {module.ModuleName}...", async _ => await module.ExecuteAsync());
        }
        catch (Exception ex)
        {
            sw.Stop();
            Log.Error(ex, "Module execution failed: {ModuleName}", module.ModuleName);
            AnsiConsole.MarkupLine($"[red]Module '{Markup.Escape(module.ModuleName)}' failed: {Markup.Escape(ex.Message)}[/]");
            AnsiConsole.MarkupLine($"[dim]Elapsed:[/] {sw.Elapsed}");
            AnsiConsole.MarkupLine($"[dim]Log:[/] {Markup.Escape(logFile)}");
            continue;
        }
        sw.Stop();

        var color = result.Status == PrivacyEnforcerPro.Core.Enums.OperationStatus.Failure ? "red" : result.Status == PrivacyEnforcerPro.Core.Enums.OperationStatus.Warning ? "yellow" : "green";
        var statusEmoji = result.Status == PrivacyEnforcerPro.Core.Enums.OperationStatus.Failure ? "❌" : result.Status == PrivacyEnforcerPro.Core.Enums.OperationStatus.Warning ? "⚠️" : "✅";
        AnsiConsole.MarkupLine($"[{color}]{statusEmoji} {Markup.Escape(result.Message)}[/]");
        AnsiConsole.MarkupLine($"[dim]Elapsed:[/] {sw.Elapsed}");
        AnsiConsole.MarkupLine($"[dim]Log:[/] {Markup.Escape(logFile)}");
    }
}
