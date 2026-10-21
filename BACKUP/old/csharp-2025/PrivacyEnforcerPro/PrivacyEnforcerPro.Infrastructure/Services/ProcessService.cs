using System.Diagnostics;
using PrivacyEnforcerPro.Core.Models;

namespace PrivacyEnforcerPro.Infrastructure.Services;

public sealed class ProcessService
{
    public async Task<OperationResult> RunAsync(string fileName, string arguments)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = fileName,
                Arguments = arguments,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
                CreateNoWindow = true
            };

            using var process = new Process { StartInfo = psi };
            process.Start();
            var output = await process.StandardOutput.ReadToEndAsync();
            var error = await process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();

            return process.ExitCode == 0
                ? OperationResult.Success(output)
                : OperationResult.Failure(string.IsNullOrWhiteSpace(error) ? output : error);
        }
        catch (Exception ex)
        {
            return OperationResult.Failure(ex.Message);
        }
    }
}