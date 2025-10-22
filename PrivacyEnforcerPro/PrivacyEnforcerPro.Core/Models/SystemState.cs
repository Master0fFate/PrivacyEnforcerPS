namespace PrivacyEnforcerPro.Core.Models;

public sealed class SystemState
{
    public DateTime TimestampUtc { get; init; } = DateTime.UtcNow;
    public string OsVersion { get; init; } = Environment.OSVersion.VersionString;
    public Dictionary<string, object?> Snapshot { get; init; } = new();
}