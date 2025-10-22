using System.Security.Cryptography;

namespace PrivacyEnforcerPro.Infrastructure.Security;

public sealed class SecureCredentialManager
{
    public byte[] Protect(byte[] data) => ProtectedData.Protect(data, null, DataProtectionScope.LocalMachine);
    public byte[] Unprotect(byte[] data) => ProtectedData.Unprotect(data, null, DataProtectionScope.LocalMachine);
}