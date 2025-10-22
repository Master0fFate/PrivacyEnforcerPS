using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using PrivacyEnforcerPro.Core.Interfaces;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class EncryptedConfigurationService : IConfigurationService
{
    private readonly string _appDataPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData), "PrivacyEnforcerPro");
    private readonly string _keyPath;

    public EncryptedConfigurationService()
    {
        Directory.CreateDirectory(_appDataPath);
        _keyPath = Path.Combine(_appDataPath, "config.key");
    }

    private byte[] LoadOrGenerateKey()
    {
        if (File.Exists(_keyPath))
        {
            return File.ReadAllBytes(_keyPath);
        }

        var key = RandomNumberGenerator.GetBytes(32);
        File.WriteAllBytes(_keyPath, key);
        return key;
    }

    public async Task<T?> LoadAsync<T>(string path, CancellationToken ct = default) where T : class
    {
        var key = LoadOrGenerateKey();
        var encrypted = await File.ReadAllBytesAsync(path, ct);
        var decrypted = ProtectedData.Unprotect(encrypted, key, DataProtectionScope.LocalMachine);
        var json = Encoding.UTF8.GetString(decrypted);
        return JsonSerializer.Deserialize<T>(json);
    }

    public async Task SaveAsync<T>(T config, string path, CancellationToken ct = default) where T : class
    {
        var key = LoadOrGenerateKey();
        var json = JsonSerializer.Serialize(config, new JsonSerializerOptions { WriteIndented = true });
        var plaintext = Encoding.UTF8.GetBytes(json);
        var encrypted = ProtectedData.Protect(plaintext, key, DataProtectionScope.LocalMachine);
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        await File.WriteAllBytesAsync(path, encrypted, ct);
    }
}