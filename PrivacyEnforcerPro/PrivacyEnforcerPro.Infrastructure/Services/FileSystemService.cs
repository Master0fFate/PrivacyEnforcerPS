namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class FileSystemService
{
    public string BackupFile(string sourcePath, string backupDirectory)
    {
        Directory.CreateDirectory(backupDirectory);
        var timestamp = DateTime.UtcNow.ToString("yyyyMMdd_HHmmss");
        var fileName = Path.GetFileName(sourcePath);
        var backupPath = Path.Combine(backupDirectory, $"{fileName}.{timestamp}.bak");
        File.Copy(sourcePath, backupPath, overwrite: true);
        return backupPath;
    }

    public void EnsureDirectory(string path)
    {
        Directory.CreateDirectory(path);
    }
}