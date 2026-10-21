namespace PrivacyEnforcerPro.Core.Interfaces;

public interface IConfigurationService
{
    Task<T?> LoadAsync<T>(string path, CancellationToken ct = default) where T : class;
    Task SaveAsync<T>(T config, string path, CancellationToken ct = default) where T : class;
}