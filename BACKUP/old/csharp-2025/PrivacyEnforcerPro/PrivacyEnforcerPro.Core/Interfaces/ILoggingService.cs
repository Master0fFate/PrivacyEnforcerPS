namespace PrivacyEnforcerPro.Core.Interfaces;

public interface ILoggingService
{
    void Information(string message);
    void Warning(string message);
    void Error(string message, Exception? ex = null);
}