using System.Security.Principal;

namespace PrivacyEnforcerPro.Infrastructure.Security;

public static class PrivilegeElevation
{
    public static bool IsAdministrator()
    {
        var identity = WindowsIdentity.GetCurrent();
        var principal = new WindowsPrincipal(identity);
        return principal.IsInRole(WindowsBuiltInRole.Administrator);
    }
}