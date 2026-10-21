namespace PrivacyEnforcerPro.Core.Interfaces;

public interface IAuditService
{
    Task WriteAuditAsync(string action, string details, CancellationToken ct = default);
}