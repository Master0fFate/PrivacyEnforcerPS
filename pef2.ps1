<#
.SYNOPSIS
    Privacy Enforcer
.DESCRIPTION
    Provides functionality for IP blocking, file management, and cache cleaning
.NOTES
    Version: 2.0
#>

# Admin check function
function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Initialize environment
function Initialize-Environment {
    $script:ProgressPreference = 'SilentlyContinue'
    $script:ErrorActionPreference = 'SilentlyContinue'
    $script:WarningPreference = 'SilentlyContinue'
    
    Set-ExecutionPolicy -ExecutionPolicy Bypass -Force -Confirm:$false
    
    # Set window title
    $username = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
    $osInfo = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion").ProductName
    $date = Get-Date
    $host.ui.RawUI.WindowTitle = "Privacy Enforcer [v2] | User: $username | OS: $osInfo | $date"
}

# Check PowerShell version
$psVersion = $PSVersionTable.PSVersion.Major
Write-Host "Current PowerShell version: $psVersion"

if ($psVersion -lt 7) {
    Write-Host "PowerShell 7 is not installed. Checking for Chocolatey..." -ForegroundColor Red

    # Check if Chocolatey is installed
    if (-not (Test-CommandExists choco)) {
        Write-Host "Chocolatey is not installed. Installing Chocolatey..." -ForegroundColor Red
        try {
            # Install Chocolatey
            Set-ExecutionPolicy Bypass -Scope Process -Force
            [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
            Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
            
            # Reload PATH
            $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
            
            Write-Host "Chocolatey installed successfully!" -ForegroundColor Green
        }
        catch {
            Write-Error "Failed to install Chocolatey: $_" -ForegroundColor Red
            exit 1
        }
    }
    else {
        Write-Host "Chocolatey is already installed." -ForegroundColor Yellow
    }

    # Install PowerShell 7
    Write-Host "Installing PowerShell 7..." 
    try {
        choco install powershell-core -y
        Write-Host "PowerShell 7 installed successfully!"
        Write-Host "Please restart your computer to complete the installation." 
    }
    catch {
        Write-Error "Failed to install PowerShell 7: $_" -ForegroundColor Red
        exit 1
    }
}
else {
    Write-Host "PowerShell 7 is already installed."
}
Start-Sleep -Seconds 2


# Firewall Management Functions
function Add-FirewallBlock {
    param(
        [string]$DisplayName,
        [string]$RemoteAddress
    )
    
    try {
        New-NetFirewallRule -DisplayName $DisplayName -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress $RemoteAddress
        New-NetFirewallRule -DisplayName $DisplayName -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress $RemoteAddress
        Write-Host "Successfully blocked $RemoteAddress" -ForegroundColor Green
    }
    catch {
        Write-Host "Failed to block $RemoteAddress" -ForegroundColor Red
    }
}

function Remove-FirewallBlock {
    param([string]$DisplayName)
    
    try {
        Remove-NetFirewallRule -DisplayName $DisplayName
        Write-Host "Processed rule: $DisplayName" -ForegroundColor Green
    }
    catch {
        Write-Host "Unexpected error occured" -ForegroundColor Red
    }
}

# File Management Functions
function Search-File {
    param(
        [string]$Drive,
        [string]$SearchTerm
    )
    
    $driveLetter = $Drive.Replace(":", "")
    Start-Process cmd -verb runas -ArgumentList "/k @echo off && title FindFile && dir ${driveLetter}:\*$SearchTerm* /n /s /p /b && cd/"
}

function Remove-File {
    param([string]$FilePath)
    
    try {
        Remove-Item -Path $FilePath -Force
        Write-Host "Successfully deleted: $FilePath" -ForegroundColor Green
    }
    catch {
        Write-Host "Failed to delete file" -ForegroundColor Red
    }
}

function Lock-Folder {
    param([string]$FolderPath)
    
    try {
        cacls $FolderPath /P everyone:n
        Write-Host "Folder locked successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "Failed to lock folder" -ForegroundColor Red
    }
}

function Unlock-Folder {
    param([string]$FolderPath)
    
    try {
        cacls $FolderPath /P everyone:f
        Write-Host "Folder unlocked successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "Failed to unlock folder" -ForegroundColor Red
    }
}

# System Cleaning Functions
function Clear-SystemCache {
    try {
        # Google Chrome
        Write-Host "Clearing Chrome cache..." -ForegroundColor Cyan
        Remove-Item -Path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache\*" -Recurse -Force
        Remove-Item -Path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache2\entries\*" -Recurse -Force
        Remove-Item -Path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cookies" -Recurse -Force
        Remove-Item -Path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Media Cache" -Recurse -Force
        Remove-Item -Path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cookies-Journal" -Recurse -Force

        # Mozilla Firefox
        Write-Host "Clearing Firefox cache..." -ForegroundColor Cyan
        $FirefoxProfiles = "$env:APPDATA\Mozilla\Firefox\Profiles"
        if (Test-Path $FirefoxProfiles) {
            Get-ChildItem -Path $FirefoxProfiles -Directory | ForEach-Object {
                Remove-Item -Path "$($_.FullName)\cache2\entries\*" -Recurse -Force
                Remove-Item -Path "$($_.FullName)\cache\*" -Recurse -Force
                Remove-Item -Path "$($_.FullName)\cookies.sqlite" -Force
                Remove-Item -Path "$($_.FullName)\webappsstore.sqlite" -Force
            }
        }

        # Microsoft Edge
        Write-Host "Clearing Edge cache..." -ForegroundColor Cyan
        Remove-Item -Path "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Default\Cache\*" -Recurse -Force
        Remove-Item -Path "$env:LOCALAPPDATA\Microsoft\Edge\User Data\Default\Cookies" -Force

        # Opera
        Write-Host "Clearing Opera cache..." -ForegroundColor Cyan
        Remove-Item -Path "$env:APPDATA\Opera Software\Opera Stable\Cache\*" -Recurse -Force
        Remove-Item -Path "$env:APPDATA\Opera Software\Opera Stable\Cookies" -Force

        # Safari (if installed)
        Write-Host "Clearing Safari cache..." -ForegroundColor Cyan
        $SafariPath = "$env:APPDATA\Apple Computer\Safari"
        if (Test-Path $SafariPath) {
            Remove-Item -Path "$SafariPath\Cache.db" -Force
            Remove-Item -Path "$SafariPath\WebpageIcons.db" -Force
            Remove-Item -Path "$SafariPath\Cookies.db" -Force
        }

        # Recent Items
        Write-Host "Clearing Recent Items..." -ForegroundColor Cyan
        Remove-Item -Path "$env:APPDATA\Microsoft\Windows\Recent\*" -Recurse -Force
        Remove-Item -Path "$env:APPDATA\Microsoft\Windows\Recent\AutomaticDestinations\*" -Recurse -Force
        Remove-Item -Path "$env:APPDATA\Microsoft\Windows\Recent\CustomDestinations\*" -Recurse -Force

        # Prefetch
        Write-Host "Clearing Prefetch..." -ForegroundColor Cyan
        Remove-Item -Path "C:\Windows\Prefetch\*" -Recurse -Force

        # Temp Folders
        Write-Host "Clearing Temp folders..." -ForegroundColor Cyan
        Remove-Item -Path "$env:TEMP\*" -Recurse -Force
        Remove-Item -Path "C:\Windows\Temp\*" -Recurse -Force
        Remove-Item -Path "$env:USERPROFILE\AppData\Local\Temp\*" -Recurse -Force

        # All .log files (system-wide)
        Write-Host "Clearing log files..." -ForegroundColor Cyan
        Get-ChildItem -Path "C:\" -Recurse -Filter "*.log" -Force -ErrorAction SilentlyContinue | 
        Where-Object { !$_.PSIsContainer } |
        ForEach-Object {
            try {
                Remove-Item -Path $_.FullName -Force
            }
            catch {
                # Silently continue if a log file is in use
            }
        }

        # DNS Cache
        Write-Host "Clearing DNS Cache..." -ForegroundColor Cyan
        ipconfig /flushdns | Out-Null

        Write-Host "`nAll caches cleared successfully!" -ForegroundColor Green
    }
    catch {
        Write-Host "Failed to clear some items: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}



# Menu Functions
function Show-MainMenu {
    Clear-Host
    Write-Host "=== Privacy Enforcer ===" -ForegroundColor Cyan
    Write-Host "1. IP Management"
    Write-Host "2. File Management"
    Write-Host "3. System Cleaning"
    Write-Host "4. View Credits"
    Write-Host "Q. Exit"
    Write-Host ""
}

function Show-IPMenu {
    Clear-Host
    Write-Host "=== IP Management ===" -ForegroundColor Cyan
    Write-Host "1. Block IP Address"
    Write-Host "2. Unblock IP Address"
    Write-Host "3. View Blocked IPs"
    Write-Host "B. Back to Main Menu"
    Write-Host ""
}

function Show-FileMenu {
    Clear-Host
    Write-Host "=== File Management ===" -ForegroundColor Cyan
    Write-Host "1. Search for File"
    Write-Host "2. Delete File"
    Write-Host "3. Lock Folder"
    Write-Host "4. Unlock Folder"
    Write-Host "B. Back to Main Menu"
    Write-Host ""
}

function Show-Credits {
    Clear-Host
    Write-Host "  ______   ______     ______  " -ForegroundColor Cyan
    Write-Host " /\  == \ /\  ___\   /\  ___\ " -ForegroundColor Cyan
    Write-Host " \ \  _-/ \ \  __\   \ \  __\ " -ForegroundColor Cyan
    Write-Host "  \ \_\    \ \_____\  \ \_\   " -ForegroundColor Cyan
    Write-Host "   \/_/     \/_____/   \/_/   " -ForegroundColor Cyan
    Write-Host "------------------------------" -ForegroundColor Cyan
    Write-Host "Privacy Enforcer"
	Write-Host "- this script is now barebones -"
    Write-Host ""
    Write-Host "Credits:" -ForegroundColor White
    Write-Host "github.com/Master0fFate"
    Write-Host ""
    Write-Host "honestly if you don't like gettings screenshared just dont play these games, sometimes it's not worth your privacy."
    Write-Host ""
    Write-Host "Release year: 2024"
    Write-Host ""
    Write-Host "Press any key to continue..."
    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
}

# Main script execution
if (-not (Test-Administrator)) {
    Write-Host "Please run as Administrator" -ForegroundColor Red
    Start-Sleep -Seconds 3
    exit
}

Initialize-Environment

do {
    Show-MainMenu
    $mainChoice = Read-Host "Select an option"
    
    switch ($mainChoice) {
        '1' { 
            do {
                Show-IPMenu
                $ipChoice = Read-Host "Select an option"
                switch ($ipChoice) {
                    '1' {
                        $ipAddress = Read-Host "Enter IP Address to block"
                        $displayName = Read-Host "Enter a name for this rule"
                        Add-FirewallBlock -DisplayName $displayName -RemoteAddress $ipAddress
                        pause
                    }
                    '2' {
                        $ruleName = Read-Host "Enter rule name to remove"
                        Remove-FirewallBlock -DisplayName $ruleName
                        pause
                    }
                    '3' {
                        Get-NetFirewallRule | Format-Table -Property DisplayName, Direction, Action, Enabled
                        pause
                    }
                }
            } while ($ipChoice -ne 'B')
        }
        '2' { 
            do {
                Show-FileMenu
                $fileChoice = Read-Host "Select an option"
                switch ($fileChoice) {
                    '1' {
                        $drive = Read-Host "Enter drive letter (e.g., C:)"
                        $searchTerm = Read-Host "Enter search term"
                        Search-File -Drive $drive -SearchTerm $searchTerm
                    }
                    '2' {
                        $filePath = Read-Host "Enter file path to delete"
                        Remove-File -FilePath $filePath
                        pause
                    }
                    '3' {
                        $folderPath = Read-Host "Enter folder path to lock"
                        Lock-Folder -FolderPath $folderPath
                        pause
                    }
                    '4' {
                        $folderPath = Read-Host "Enter folder path to unlock"
                        Unlock-Folder -FolderPath $folderPath
                        pause
                    }
                }
            } while ($fileChoice -ne 'B')
        }
        '3' { 
            Clear-Host
            Write-Host "Cleaning system cache..."
            Clear-SystemCache
            pause
        }
        '4' { 
            Show-Credits
        }
        'Q' { 
            Write-Host "Exiting..." -ForegroundColor Cyan
            break 
        }
        default { 
            Write-Host "Invalid option" -ForegroundColor Red
            pause
        }
    }
} while ($mainChoice -ne 'Q')
Start-Sleep -Seconds 1
stop-process -Id $PID