using PrivacyEnforcerPro.Core.Interfaces;
using Serilog;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class LoggingService : ILoggingService
{
    public void Information(string message) => Log.Information(message);
    public void Warning(string message) => Log.Warning(message);
    public void Error(string message, Exception? ex = null)
    {
        if (ex is null) Log.Error(message);
        else Log.Error(ex, message);
    }
}