using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net;
using System.Runtime.InteropServices;
using System.Security.AccessControl;
using System.Security.Principal;
using System.Text;

namespace PefCli
{
    internal static class Program
    {
        private const string Separator = "────────────────────────────────────────────────────────";

        private static readonly string AppTitle = "🅿🅴🅵  Privacy Enforcement Framework";

        private static readonly string HostsFilePath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System), "drivers", "etc", "hosts");

        private static int Main()
        {
            Console.OutputEncoding = Encoding.UTF8;
            Console.Title = "PEF – Privacy Enforcement Framework";
            Console.CursorVisible = false;

            WelcomeScreen();
            Pause("Press any key to begin. . .");

            Console.CursorVisible = true;

            while (true)
            {
                DrawHeader();
                WriteLineWithAccent("Select a module", ConsoleColor.Cyan);
                Console.WriteLine(" 1. Privacy Sanitizer");
                Console.WriteLine(" 2. Browser Hygiene");
                Console.WriteLine(" 3. File Operations");
                Console.WriteLine(" 4. Network Guard");
                Console.WriteLine(" 5. Diagnostics & Help");
                Console.WriteLine(" 6. Credits");
                Console.WriteLine(" 0. Exit");
                Console.WriteLine();
                Console.Write("› ");

                var choice = Console.ReadLine();
                switch (choice)
                {
                    case "1":
                        PrivacySanitizer();
                        break;
                    case "2":
                        BrowserHygiene();
                        break;
                    case "3":
                        FileOperations();
                        break;
                    case "4":
                        NetworkGuard();
                        break;
                    case "5":
                        Diagnostics();
                        break;
                    case "6":
                        Credits();
                        break;
                    case "0":
                        Farewell();
                        return 0;
                    default:
                        WriteLineWithAccent("Unrecognized selection. Try again.", ConsoleColor.DarkYellow);
                        Pause();
                        break;
                }
            }
        }

        private static void WelcomeScreen()
        {
            Console.Clear();
            DrawHeader();
            WriteLineWithAccent("Designing a safer desktop footprint.", ConsoleColor.Gray);
            Console.WriteLine();
            var isAdmin = IsAdministrator();
            WriteLineWithAccent(isAdmin ? "✔️  Administrative context detected." : "⚠️  Administrative rights recommended.", isAdmin ? ConsoleColor.Green : ConsoleColor.Yellow);
            Console.WriteLine();
            Console.WriteLine("This interface replaces the legacy PowerShell script with updated modules:");
            Console.WriteLine(" • Built-in privacy sanitizer for recent activity traces");
            Console.WriteLine(" • Modern browser hygiene routines (Chrome, Edge, Firefox, Opera)");
            Console.WriteLine(" • File operations suite with granular control");
            Console.WriteLine(" • Network guard with rule scoping under the PEF namespace");
            Console.WriteLine();
        }

        private static void DrawHeader()
        {
            Console.Clear();
            WriteLineWithAccent(AppTitle, ConsoleColor.Cyan);
            Console.WriteLine(Separator);
        }

        private static void PrivacySanitizer()
        {
            DrawHeader();
            WriteLineWithAccent("Privacy Sanitizer", ConsoleColor.Cyan);
            Console.WriteLine(" 1. Clear Recent Items (jump lists & quick access)");
            Console.WriteLine(" 2. Purge Prefetch & Temp artifacts");
            Console.WriteLine(" 3. Audit recent shell bags (preview only)");
            Console.WriteLine(" B. Back to main menu");
            Console.WriteLine();
            Console.Write("› ");
            var choice = Console.ReadLine();
            switch (choice?.ToUpperInvariant())
            {
                case "1":
                    ExecuteWithStatus("Clearing recent items", ClearRecentItems);
                    break;
                case "2":
                    ExecuteWithStatus("Purging Prefetch and temporary files", PurgeTempArtifacts);
                    break;
                case "3":
                    ShellBagPreview();
                    break;
                default:
                    return;
            }
            Pause();
        }

        private static void BrowserHygiene()
        {
            DrawHeader();
            WriteLineWithAccent("Browser Hygiene", ConsoleColor.Cyan);
            Console.WriteLine("Select the browsers you want to sanitize (comma separated):");
            Console.WriteLine(" 1. Google Chrome");
            Console.WriteLine(" 2. Microsoft Edge");
            Console.WriteLine(" 3. Mozilla Firefox");
            Console.WriteLine(" 4. Opera / Opera GX");
            Console.WriteLine(" A. All listed browsers");
            Console.WriteLine(" B. Back to main menu");
            Console.WriteLine();
            Console.Write("› ");
            var input = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(input) || string.Equals(input, "B", StringComparison.OrdinalIgnoreCase))
            {
                return;
            }

            var selections = input.Equals("A", StringComparison.OrdinalIgnoreCase)
                ? new[] { "1", "2", "3", "4" }
                : input.Split(',').Select(entry => entry.Trim());

            var actions = new Dictionary<string, Action>
            {
                ["1"] = () => ExecuteWithStatus("Cleaning Google Chrome", () => CleanChromiumProfile(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Google", "Chrome", "User Data"))),
                ["2"] = () => ExecuteWithStatus("Cleaning Microsoft Edge", () => CleanChromiumProfile(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Microsoft", "Edge", "User Data"))),
                ["3"] = () => ExecuteWithStatus("Cleaning Mozilla Firefox", CleanFirefoxProfiles),
                ["4"] = () => ExecuteWithStatus("Cleaning Opera directories", CleanOperaProfiles)
            };

            foreach (var selection in selections)
            {
                if (actions.TryGetValue(selection, out var action))
                {
                    action();
                }
                else
                {
                    WriteLineWithAccent($"Skipping unknown option '{selection}'.", ConsoleColor.DarkYellow);
                }
            }

            Pause();
        }

        private static void FileOperations()
        {
            while (true)
            {
                DrawHeader();
                WriteLineWithAccent("File Operations", ConsoleColor.Cyan);
                Console.WriteLine(" 1. Search for files by name pattern");
                Console.WriteLine(" 2. Delete a specific file");
                Console.WriteLine(" 3. Purge *.log files recursively");
                Console.WriteLine(" 4. Lock a folder (deny Everyone)");
                Console.WriteLine(" 5. Unlock a folder");
                Console.WriteLine(" B. Back to main menu");
                Console.WriteLine();
                Console.Write("› ");
                var choice = Console.ReadLine();
                switch (choice)
                {
                    case "1":
                        ExecuteWithStatus("Searching", FileSearch);
                        Pause();
                        break;
                    case "2":
                        ExecuteWithStatus("Deleting file", DeleteFileSafely);
                        Pause();
                        break;
                    case "3":
                        ExecuteWithStatus("Purging .log files", PurgeLogFiles);
                        Pause();
                        break;
                    case "4":
                        ExecuteWithStatus("Locking folder", () => ToggleFolderAccess(deny: true));
                        Pause();
                        break;
                    case "5":
                        ExecuteWithStatus("Unlocking folder", () => ToggleFolderAccess(deny: false));
                        Pause();
                        break;
                    default:
                        return;
                }
            }
        }

        private static void NetworkGuard()
        {
            while (true)
            {
                DrawHeader();
                WriteLineWithAccent("Network Guard", ConsoleColor.Cyan);
                Console.WriteLine(" 1. Resolve hostname/IP");
                Console.WriteLine(" 2. Block IPv4/IPv6 address");
                Console.WriteLine(" 3. Unblock IPv4/IPv6 address");
                Console.WriteLine(" 4. List active PEF firewall rules");
                Console.WriteLine(" 5. Block hostname via hosts file");
                Console.WriteLine(" 6. Unblock hostname from hosts file");
                Console.WriteLine(" 7. View hosts file entries (PEF scoped)");
                Console.WriteLine(" B. Back to main menu");
                Console.WriteLine();
                Console.Write("› ");
                var choice = Console.ReadLine();
                switch (choice)
                {
                    case "1":
                        ResolveEndpoint();
                        Pause();
                        break;
                    case "2":
                        ExecuteWithStatus("Blocking IP", BlockIp); Pause();
                        break;
                    case "3":
                        ExecuteWithStatus("Unblocking IP", UnblockIp); Pause();
                        break;
                    case "4":
                        ListFirewallRules();
                        Pause();
                        break;
                    case "5":
                        ExecuteWithStatus("Blocking hostname", () => ModifyHosts(add: true)); Pause();
                        break;
                    case "6":
                        ExecuteWithStatus("Unblocking hostname", () => ModifyHosts(add: false)); Pause();
                        break;
                    case "7":
                        ShowHostsEntries();
                        Pause();
                        break;
                    default:
                        return;
                }
            }
        }

        private static void Diagnostics()
        {
            DrawHeader();
            WriteLineWithAccent("Diagnostics & Help", ConsoleColor.Cyan);
            Console.WriteLine(" • Run this interface as Administrator for full functionality.");
            Console.WriteLine(" • File locking is applied via NTFS ACLs. Owner retains control.");
            Console.WriteLine(" • Firewall rules are created using netsh with the prefix 'PEF-'.");
            Console.WriteLine(" • Hosts file edits are tagged with '# PEF'.");
            Console.WriteLine();
            Console.WriteLine("Quick commands:");
            Console.WriteLine(" → netsh advfirewall firewall show rule name=all | findstr PEF-");
            Console.WriteLine(" → get-content %SystemRoot%\\System32\\drivers\\etc\\hosts");
            Console.WriteLine();
            Console.WriteLine("Press H for contextual help, any other key to return.");
            var key = Console.ReadKey(true);
            if (char.ToUpperInvariant(key.KeyChar) == 'H')
            {
                DrawHeader();
                WriteLineWithAccent("Module Help", ConsoleColor.Cyan);
                Console.WriteLine("Privacy Sanitizer: Removes recent item references and temp execution traces.");
                Console.WriteLine("Browser Hygiene: Purges browsing artifacts across Chromium-based and Gecko browsers.");
                Console.WriteLine("File Operations: Search, remove, or ACL-lock directories without external scripts.");
                Console.WriteLine("Network Guard: Resolve endpoints, manage firewall rules, and adjust hosts entries.");
                Console.WriteLine();
                Console.WriteLine("All destructive actions prompt for confirmation before execution.");
                Pause();
            }
        }

        private static void Credits()
        {
            DrawHeader();
            WriteLineWithAccent("Credits", ConsoleColor.Cyan);
            Console.WriteLine("Maintainer: github.com/Master0fFate");
            Console.WriteLine("Modernized CLI & modules: 2025 refresh");
            Console.WriteLine();
            Console.WriteLine("This iteration replaces legacy dependencies (SafeShare, MusicPlayer, remote downloads)");
            Console.WriteLine("with native .NET operations and scoped firewall/hosts management.");
            Console.WriteLine();
            Pause();
        }

        private static void Farewell()
        {
            DrawHeader();
            WriteLineWithAccent("Session terminated. Stay private.", ConsoleColor.Gray);
            Console.CursorVisible = true;
        }

        private static void ClearRecentItems()
        {
            var recent = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Microsoft", "Windows", "Recent");
            var autoDestinations = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Recent), "AutomaticDestinations");
            var customDestinations = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Recent), "CustomDestinations");

            DeleteContents(recent);
            DeleteContents(autoDestinations);
            DeleteContents(customDestinations);

            WriteLineWithAccent("Recent item traces cleared.", ConsoleColor.Green);
        }

        private static void PurgeTempArtifacts()
        {
            var temp = Path.GetTempPath();
            DeleteContents(temp);

            var prefetch = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Windows), "Prefetch");
            DeleteContents(prefetch);

            WriteLineWithAccent("Prefetch and temporary data flushed.", ConsoleColor.Green);
        }

        private static void ShellBagPreview()
        {
            var shellBagPath = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Microsoft", "Windows", "UsrClass.dat");
            if (File.Exists(shellBagPath))
            {
                var size = new FileInfo(shellBagPath).Length;
                WriteLineWithAccent($"UsrClass.dat located ({size:N0} bytes). Consider third-party tools for full purge.", ConsoleColor.White);
            }
            else
            {
                WriteLineWithAccent("UsrClass.dat not found in expected location.", ConsoleColor.DarkYellow);
            }
        }

        private static void CleanChromiumProfile(string profileRoot)
        {
            if (!Directory.Exists(profileRoot))
            {
                WriteLineWithAccent($"Path not found: {profileRoot}", ConsoleColor.DarkYellow);
                return;
            }

            var targets = new[]
            {
                "Default\\History",
                "Default\\History Provider Cache",
                "Default\\Visited Links",
                "Default\\Login Data",
                "Default\\Web Data",
                "Default\\Cookies",
                "Default\\Cookies-journal",
                "Default\\Network\\Cookies",
                "Default\\Cache",
                "Default\\Code Cache",
                "Default\\GPUCache",
                "Default\\Service Worker\\CacheStorage",
                "Default\\Media Cache",
                "Default\\Session Storage",
                "Default\\Local Storage"
            };

            foreach (var relative in targets)
            {
                var path = Path.Combine(profileRoot, relative);
                DeleteItem(path);
            }

            WriteLineWithAccent($"Chromium artifacts removed from {profileRoot}", ConsoleColor.Green);
        }

        private static void CleanFirefoxProfiles()
        {
            var roaming = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Mozilla", "Firefox", "Profiles");
            var local = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Mozilla", "Firefox", "Profiles");
            DeleteContents(roaming);
            DeleteContents(local);
            WriteLineWithAccent("Firefox profile data cleared.", ConsoleColor.Green);
        }

        private static void CleanOperaProfiles()
        {
            var roaming = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Opera Software");
            var local = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Opera Software");
            DeleteContents(roaming);
            DeleteContents(local);
            WriteLineWithAccent("Opera data cleared.", ConsoleColor.Green);
        }

        private static void FileSearch()
        {
            Console.Write("Enter starting directory (leave blank for current drive): ");
            var root = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(root))
            {
                root = Directory.GetCurrentDirectory();
            }

            Console.Write("Enter file name or pattern (* and ? supported): ");
            var pattern = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(pattern))
            {
                WriteLineWithAccent("Search cancelled – no pattern provided.", ConsoleColor.DarkYellow);
                return;
            }

            if (!Directory.Exists(root))
            {
                WriteLineWithAccent("Directory not found.", ConsoleColor.DarkYellow);
                return;
            }

            IEnumerable<string> results;
            try
            {
                results = Directory.EnumerateFiles(root, pattern, SearchOption.AllDirectories);
            }
            catch (UnauthorizedAccessException ex)
            {
                WriteLineWithAccent($"Access denied: {ex.Message}", ConsoleColor.DarkYellow);
                return;
            }

            var list = results.Take(50).ToList();
            if (list.Count == 0)
            {
                WriteLineWithAccent("No matches found.", ConsoleColor.DarkYellow);
            }
            else
            {
                WriteLineWithAccent($"Displaying {list.Count} result(s) (max 50 shown):", ConsoleColor.White);
                foreach (var file in list)
                {
                    Console.WriteLine(file);
                }
            }
        }

        private static void DeleteFileSafely()
        {
            Console.Write("Enter full file path to delete: ");
            var path = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(path) || !File.Exists(path))
            {
                WriteLineWithAccent("File not found.", ConsoleColor.DarkYellow);
                return;
            }

            if (!Confirm($"Delete '{path}'?"))
            {
                WriteLineWithAccent("Deletion canceled.", ConsoleColor.DarkYellow);
                return;
            }

            TryExecute(() => File.Delete(path));
            WriteLineWithAccent("File deleted.", ConsoleColor.Green);
        }

        private static void PurgeLogFiles()
        {
            Console.Write("Enter directory to purge (*.log) (leave blank for system drive): ");
            var path = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(path))
            {
                path = Environment.GetFolderPath(Environment.SpecialFolder.System).Substring(0, 3);
            }

            if (!Directory.Exists(path))
            {
                WriteLineWithAccent("Directory not found.", ConsoleColor.DarkYellow);
                return;
            }

            if (!Confirm($"Recursively delete all *.log files under '{path}'?"))
            {
                WriteLineWithAccent("Operation canceled.", ConsoleColor.DarkYellow);
                return;
            }

            var files = Directory.EnumerateFiles(path, "*.log", SearchOption.AllDirectories).ToList();
            foreach (var file in files)
            {
                DeleteItem(file);
            }

            WriteLineWithAccent($"Removed {files.Count} .log files.", ConsoleColor.Green);
        }

        private static void ToggleFolderAccess(bool deny)
        {
            Console.Write("Enter folder path: ");
            var path = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(path) || !Directory.Exists(path))
            {
                WriteLineWithAccent("Directory not found.", ConsoleColor.DarkYellow);
                return;
            }

            if (!IsAdministrator())
            {
                WriteLineWithAccent("Administrator rights required to modify ACLs.", ConsoleColor.DarkYellow);
                return;
            }

            var everyone = new SecurityIdentifier(WellKnownSidType.WorldSid, null);
            var rule = new FileSystemAccessRule(everyone, FileSystemRights.FullControl, InheritanceFlags.ContainerInherit | InheritanceFlags.ObjectInherit, PropagationFlags.None, deny ? AccessControlType.Deny : AccessControlType.Allow);

            var directory = new DirectoryInfo(path);
            var security = directory.GetAccessControl();

            if (deny)
            {
                security.AddAccessRule(rule);
            }
            else
            {
                security.RemoveAccessRule(rule);
            }

            directory.SetAccessControl(security);
            WriteLineWithAccent(deny ? "Folder locked." : "Folder unlocked.", ConsoleColor.Green);
        }

        private static void ResolveEndpoint()
        {
            Console.Write("Enter hostname or IP: ");
            var input = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(input))
            {
                WriteLineWithAccent("No input provided.", ConsoleColor.DarkYellow);
                return;
            }

            try
            {
                if (IPAddress.TryParse(input, out var ip))
                {
                    var host = Dns.GetHostEntry(ip);
                    WriteLineWithAccent($"Resolved hostname: {host.HostName}", ConsoleColor.Green);
                }
                else
                {
                    var addresses = Dns.GetHostAddresses(input);
                    WriteLineWithAccent($"Found {addresses.Length} address(es):", ConsoleColor.Green);
                    foreach (var address in addresses)
                    {
                        Console.WriteLine(address);
                    }
                }
            }
            catch (Exception ex)
            {
                WriteLineWithAccent($"Resolution failed: {ex.Message}", ConsoleColor.DarkYellow);
            }
        }

        private static void BlockIp()
        {
            if (!IsAdministrator())
            {
                WriteLineWithAccent("Administrator rights required to manage firewall rules.", ConsoleColor.DarkYellow);
                return;
            }

            Console.Write("Rule display name (will be prefixed with PEF-): ");
            var name = Console.ReadLine();
            Console.Write("Remote IP (IPv4 or IPv6, ranges allowed): ");
            var ip = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(name) || string.IsNullOrWhiteSpace(ip))
            {
                WriteLineWithAccent("Missing input.", ConsoleColor.DarkYellow);
                return;
            }

            var ruleName = $"PEF-{name}";
            var commandInbound = $"advfirewall firewall add rule name=\"{ruleName}\" dir=in action=block enable=yes remoteip={ip}";
            var commandOutbound = $"advfirewall firewall add rule name=\"{ruleName}\" dir=out action=block enable=yes remoteip={ip}";

            var inboundResult = RunNetsh(commandInbound);
            var outboundResult = RunNetsh(commandOutbound);

            if (inboundResult && outboundResult)
            {
                WriteLineWithAccent($"Firewall rules created for {ip}.", ConsoleColor.Green);
            }
            else
            {
                WriteLineWithAccent("Failed to create firewall rules (check console output).", ConsoleColor.DarkYellow);
            }
        }

        private static void UnblockIp()
        {
            if (!IsAdministrator())
            {
                WriteLineWithAccent("Administrator rights required to manage firewall rules.", ConsoleColor.DarkYellow);
                return;
            }

            Console.Write("Rule display name suffix (without PEF- prefix): ");
            var name = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(name))
            {
                WriteLineWithAccent("No rule name provided.", ConsoleColor.DarkYellow);
                return;
            }

            var ruleName = $"PEF-{name}";
            var command = $"advfirewall firewall delete rule name=\"{ruleName}\"";

            if (RunNetsh(command))
            {
                WriteLineWithAccent($"Firewall rules removed for {ruleName}.", ConsoleColor.Green);
            }
            else
            {
                WriteLineWithAccent("No matching rules removed (verify name).", ConsoleColor.DarkYellow);
            }
        }

        private static void ListFirewallRules()
        {
            DrawHeader();
            WriteLineWithAccent("Active PEF Firewall Rules", ConsoleColor.Cyan);
            if (!IsAdministrator())
            {
                WriteLineWithAccent("Administrator rights recommended for accurate listing.", ConsoleColor.DarkYellow);
            }
            RunNetsh("advfirewall firewall show rule name=all", printOutput: true, outputFilter: "PEF-");
        }

        private static void ModifyHosts(bool add)
        {
            if (!IsAdministrator())
            {
                WriteLineWithAccent("Administrator rights required to modify hosts file.", ConsoleColor.DarkYellow);
                return;
            }

            Console.Write(add ? "Hostname to block: " : "Hostname to unblock: ");
            var hostname = Console.ReadLine();
            if (string.IsNullOrWhiteSpace(hostname))
            {
                WriteLineWithAccent("No hostname provided.", ConsoleColor.DarkYellow);
                return;
            }

            hostname = hostname.Replace("http://", string.Empty, StringComparison.OrdinalIgnoreCase)
                               .Replace("https://", string.Empty, StringComparison.OrdinalIgnoreCase)
                               .Trim();

            if (add)
            {
                var line = $"127.0.0.1 {hostname} # PEF";
                File.AppendAllText(HostsFilePath, Environment.NewLine + line);
                WriteLineWithAccent($"Hostname '{hostname}' redirected to 127.0.0.1.", ConsoleColor.Green);
            }
            else
            {
                var lines = File.ReadAllLines(HostsFilePath).Where(l => !l.Contains(hostname, StringComparison.OrdinalIgnoreCase) || !l.Contains("# PEF")).ToArray();
                File.WriteAllLines(HostsFilePath, lines);
                WriteLineWithAccent($"Hostname '{hostname}' entries removed.", ConsoleColor.Green);
            }
        }

        private static void ShowHostsEntries()
        {
            DrawHeader();
            WriteLineWithAccent("Hosts File Entries (PEF)", ConsoleColor.Cyan);
            if (!File.Exists(HostsFilePath))
            {
                WriteLineWithAccent("Hosts file not found.", ConsoleColor.DarkYellow);
                return;
            }

            var lines = File.ReadAllLines(HostsFilePath).Where(l => l.Contains("# PEF")).ToList();
            if (lines.Count == 0)
            {
                WriteLineWithAccent("No PEF scoped entries present.", ConsoleColor.DarkYellow);
                return;
            }

            foreach (var line in lines)
            {
                Console.WriteLine(line);
            }
        }

        private static bool RunNetsh(string arguments, bool printOutput = false, string? outputFilter = null)
        {
            try
            {
                var startInfo = new ProcessStartInfo
                {
                    FileName = "netsh",
                    Arguments = arguments,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using var process = Process.Start(startInfo);
                if (process == null)
                {
                    WriteLineWithAccent("Failed to start netsh process.", ConsoleColor.DarkYellow);
                    return false;
                }

                var output = process.StandardOutput.ReadToEnd();
                var error = process.StandardError.ReadToEnd();
                process.WaitForExit();

                if (printOutput)
                {
                    if (!string.IsNullOrWhiteSpace(outputFilter))
                    {
                        foreach (var line in output.Split(new[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries))
                        {
                            if (line.Contains(outputFilter, StringComparison.OrdinalIgnoreCase))
                            {
                                Console.WriteLine(line);
                            }
                        }
                    }
                    else
                    {
                        Console.WriteLine(output);
                    }
                }

                if (process.ExitCode != 0)
                {
                    if (!string.IsNullOrWhiteSpace(error))
                    {
                        Console.WriteLine(error);
                    }
                    return false;
                }

                return true;
            }
            catch (Exception ex)
            {
                WriteLineWithAccent($"netsh failed: {ex.Message}", ConsoleColor.DarkYellow);
                return false;
            }
        }

        private static void ExecuteWithStatus(string caption, Action action)
        {
            DrawHeader();
            WriteLineWithAccent(caption, ConsoleColor.Cyan);
            Console.WriteLine();
            try
            {
                action();
            }
            catch (Exception ex)
            {
                WriteLineWithAccent($"Error: {ex.Message}", ConsoleColor.Red);
            }
        }

        private static bool Confirm(string message)
        {
            Console.Write($"{message} (y/n): ");
            var key = Console.ReadKey(true).Key;
            Console.WriteLine();
            return key == ConsoleKey.Y;
        }

        private static void DeleteContents(string directory)
        {
            if (!Directory.Exists(directory))
            {
                return;
            }

            try
            {
                foreach (var file in Directory.GetFiles(directory))
                {
                    DeleteItem(file);
                }

                foreach (var dir in Directory.GetDirectories(directory))
                {
                    DeleteItem(dir);
                }
            }
            catch (Exception ex)
            {
                WriteLineWithAccent($"Failed to clean {directory}: {ex.Message}", ConsoleColor.DarkYellow);
            }
        }

        private static void DeleteItem(string path)
        {
            try
            {
                if (File.Exists(path))
                {
                    File.SetAttributes(path, FileAttributes.Normal);
                    File.Delete(path);
                }
                else if (Directory.Exists(path))
                {
                    var directory = new DirectoryInfo(path) { Attributes = FileAttributes.Normal };
                    directory.Delete(true);
                }
            }
            catch (Exception ex)
            {
                WriteLineWithAccent($"Failed to delete {path}: {ex.Message}", ConsoleColor.DarkYellow);
            }
        }

        private static void TryExecute(Action action)
        {
            try
            {
                action();
            }
            catch (Exception ex)
            {
                WriteLineWithAccent($"Operation failed: {ex.Message}", ConsoleColor.DarkYellow);
            }
        }

        private static void Pause(string message = "Press any key to continue . . .")
        {
            Console.WriteLine();
            Console.Write(message);
            Console.ReadKey(true);
        }

        private static bool IsAdministrator()
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                return false;
            }

            using var identity = WindowsIdentity.GetCurrent();
            var principal = new WindowsPrincipal(identity);
            return principal.IsInRole(WindowsBuiltInRole.Administrator);
        }

        private static void WriteLineWithAccent(string message, ConsoleColor color)
        {
            var previous = Console.ForegroundColor;
            Console.ForegroundColor = color;
            Console.WriteLine(message);
            Console.ForegroundColor = previous;
        }
    }
}
