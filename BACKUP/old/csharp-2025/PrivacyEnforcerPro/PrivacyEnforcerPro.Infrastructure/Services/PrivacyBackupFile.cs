using System.Text.Json;
using System.Text.Json.Serialization;
using PrivacyEnforcerPro.Core.Policies;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public static class PrivacyBackupFile
{
    public static readonly JsonSerializerOptions Json = new() { WriteIndented = true, UnmappedMemberHandling = JsonUnmappedMemberHandling.Disallow };
    public static void Save(string path, PrivacyBackup backup)
    {
        // Exclusive create prevents silently replacing an earlier recovery point.
        using var stream = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None);
        JsonSerializer.Serialize(stream, backup, Json);
        stream.Flush(flushToDisk: true);
    }
    public static PrivacyBackup Load(string path)
    {
        using var stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read);
        if (stream.Length > 64 * 1024) throw new ArgumentException("Backup exceeds 64 KiB limit.");
        using var document = JsonDocument.Parse(stream);
        RejectDuplicateProperties(document.RootElement);
        return document.RootElement.Deserialize<PrivacyBackup>(Json) ?? throw new ArgumentException("Empty backup.");
    }
    private static void RejectDuplicateProperties(JsonElement element)
    {
        if (element.ValueKind == JsonValueKind.Object)
        {
            var names = new HashSet<string>(StringComparer.Ordinal);
            foreach (var property in element.EnumerateObject())
            {
                if (!names.Add(property.Name)) throw new JsonException($"Duplicate property: {property.Name}");
                RejectDuplicateProperties(property.Value);
            }
        }
        else if (element.ValueKind == JsonValueKind.Array)
            foreach (var child in element.EnumerateArray()) RejectDuplicateProperties(child);
    }
}
