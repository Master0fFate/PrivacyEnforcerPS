using System.Security.Cryptography;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class FileOperationsService
{
    public async Task<List<FileInfo>> SearchFilesAsync(string rootPath, string pattern, SearchOption searchOption = SearchOption.AllDirectories)
    {
        var results = new List<FileInfo>();
        var regex = new System.Text.RegularExpressions.Regex(pattern, System.Text.RegularExpressions.RegexOptions.IgnoreCase);
        await Task.Run(() =>
        {
            var options = new EnumerationOptions
            {
                RecurseSubdirectories = searchOption == SearchOption.AllDirectories,
                IgnoreInaccessible = true,
                ReturnSpecialDirectories = false
            };
            try
            {
                foreach (var file in Directory.EnumerateFiles(rootPath, "*", options))
                {
                    if (regex.IsMatch(Path.GetFileName(file)))
                        results.Add(new FileInfo(file));
                }
            }
            catch
            {
                // Skip roots that are inaccessible
            }
        });
        return results;
    }

    public async Task SecureDeleteAsync(string path, int passes = 3)
    {
        if (!File.Exists(path)) return;
        var length = new FileInfo(path).Length;
        using var fs = new FileStream(path, FileMode.Open, FileAccess.Write, FileShare.None);
        var buffer = new byte[8192];
        for (int p = 0; p < passes; p++)
        {
            fs.Position = 0;
            long written = 0;
            while (written < length)
            {
                RandomNumberGenerator.Fill(buffer);
                var toWrite = (int)Math.Min(buffer.Length, length - written);
                await fs.WriteAsync(buffer.AsMemory(0, toWrite));
                written += toWrite;
            }
            await fs.FlushAsync();
        }
        fs.Close();
        File.Delete(path);
    }

    public int PurgeLogDirectories(IEnumerable<string> directories, int olderThanDays = 0)
    {
        int deleted = 0;
        var options = new EnumerationOptions
        {
            RecurseSubdirectories = true,
            IgnoreInaccessible = true,
            ReturnSpecialDirectories = false
        };
        foreach (var dir in directories)
        {
            if (!Directory.Exists(dir)) continue;
            try
            {
                // Skip known protected ETW/WMI real-time trace folder explicitly
                if (dir.Contains("\\RtBackup", StringComparison.OrdinalIgnoreCase))
                    continue;

                foreach (var file in Directory.EnumerateFiles(dir, "*.*", options))
                {
                    try
                    {
                        var fi = new FileInfo(file);
                        if (olderThanDays > 0 && fi.LastWriteTimeUtc > DateTime.UtcNow.AddDays(-olderThanDays))
                            continue;

                        var ext = fi.Extension;
                        if (string.Equals(ext, ".log", StringComparison.OrdinalIgnoreCase) ||
                            string.Equals(ext, ".etl", StringComparison.OrdinalIgnoreCase) ||
                            string.Equals(ext, ".evtx", StringComparison.OrdinalIgnoreCase))
                        {
                            File.Delete(file);
                            deleted++;
                        }
                    }
                    catch
                    {
                        // Ignore failures on individual files
                    }
                }
            }
            catch
            {
                // Directory enumeration not permitted; skip and continue
            }
        }
        return deleted;
    }
}