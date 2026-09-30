using System.Text.Json.Serialization;

namespace PrivacyEnforcerPro.Core.Policies;

public sealed record PrivacyPolicy(string Id, string Key, string Name, int Desired, string Impact, string Source);
public sealed record PolicyValue([property: JsonRequired] bool Exists, [property: JsonRequired] int Value = 0);
public sealed record PolicyChange([property: JsonRequired] string Id, [property: JsonRequired] PolicyValue Before, [property: JsonRequired] int Desired);
public sealed record PrivacyBackup([property: JsonRequired] int Version, [property: JsonRequired] string Machine, [property: JsonRequired] DateTimeOffset Created, [property: JsonRequired] List<PolicyChange> Changes);
public sealed record AuditFinding(string Id, string State, PolicyValue? Current, int Desired, string Impact, string? Error = null);

public interface IPolicyStore
{
    PolicyValue Read(PrivacyPolicy policy);
    void Write(PrivacyPolicy policy, PolicyValue value);
}

public static class PolicyCatalog
{
    private const string Root = @"SOFTWARE\Policies\Microsoft\Windows\";
    private const string Source = "https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-privacy#";
    public static IReadOnlyList<PrivacyPolicy> All { get; } = Array.AsReadOnly(new[]
    {
        new PrivacyPolicy("advertising-id", Root + "AdvertisingInfo", "DisabledByGroupPolicy", 1, "Stops apps using the Windows advertising ID; does not block all advertising or tracking.", Source + "disableadvertisingid"),
        new PrivacyPolicy("clipboard-sync", Root + "System", "AllowCrossDeviceClipboard", 0, "Prevents clipboard synchronization to other devices; local copy/paste remains available.", Source + "allowcrossdeviceclipboard"),
        new PrivacyPolicy("activity-feed", Root + "System", "EnableActivityFeed", 0, "Restricts activity feed publishing and roaming; does not erase previously collected data.", Source + "enableactivityfeed"),
        new PrivacyPolicy("online-speech", @"SOFTWARE\Policies\Microsoft\InputPersonalization", "AllowInputPersonalization", 0, "Disables cloud speech recognition and may prevent voice dictation.", Source + "allowinputpersonalization"),
        new PrivacyPolicy("app-camera", Root + "AppPrivacy", "LetAppsAccessCamera", 2, "Denies camera access to covered Windows apps; video calls can break. Per-app policy overrides and desktop apps need separate review.", Source + "letappsaccesscamera"),
        new PrivacyPolicy("app-microphone", Root + "AppPrivacy", "LetAppsAccessMicrophone", 2, "Denies microphone access to covered Windows apps; calls and recording can break. Per-app policy overrides and desktop apps need separate review.", Source + "letappsaccessmicrophone"),
        new PrivacyPolicy("app-location", Root + "AppPrivacy", "LetAppsAccessLocation", 2, "Denies location access to covered Windows apps; maps and location features can break. Per-app policy overrides and desktop apps need separate review.", Source + "letappsaccesslocation")
    });

    public static PrivacyPolicy Get(string id) => All.SingleOrDefault(p => p.Id == id)
        ?? throw new ArgumentException($"Unknown policy '{id}'. Run list for valid IDs.");

    public static IReadOnlyList<PrivacyPolicy> Select(IEnumerable<string> ids)
    {
        var result = ids.Select(Get).DistinctBy(p => p.Id).ToArray();
        if (result.Length == 0) throw new ArgumentException("Select at least one explicit policy ID.");
        return result;
    }
}
