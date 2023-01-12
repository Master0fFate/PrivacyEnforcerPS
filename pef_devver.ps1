#if ($PSVersionTable.PSVersion.Major -ne "7") {  #remove comments from this if you do not want to use loader.bat

#Write-Host "BACKUP Main dependency installation process starting: [Powershell7]"
#winget install --id Microsoft.Powershell --source winget
#Write-Host "PowerShell 7 Installed. Starting . . ."
#Start-Sleep 3
#Start-Process pwsh.exe -Verb RunAs -ArgumentList ('-noprofile -noexit -file "{0}" -elevated' -f ($myinvocation.MyCommand.Definition)) #test
#exit }


#function Test-Admin {
#    $currentUser = New-Object Security.Principal.WindowsPrincipal $([Security.Principal.WindowsIdentity]::GetCurrent())
#    $currentUser.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)
#}

#if ((Test-Admin) -eq $false)  {
#    if ($elevated) {
#        # tried to elevate, did not work, aborting
 #   } else {
  #      Start-Process pwsh.exe -Verb RunAs -ArgumentList ('-noprofile -noexit -file "{0}" -elevated' -f ($myinvocation.MyCommand.Definition))
  #  }
  #  exit
#}
Set-ExecutionPolicy -ExecutionPolicy Bypass -Force -Confirm:$false
$global:prevProgressPreference = $global:ProgressPreference
$global:ProgressPreference = 'SilentlyContinue' #Hide Download UI
$ErrorActionPreference = 'SilentlyContinue' #| out-null for no output:) superhide stuff
$global:WarningActionPreference = 'SilentlyContinue'
$username = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
$osinfo = (Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion").ProductName
$date = Get-Date
$host.ui.RawUI.WindowTitle = "Privacy Enforcer [v1.0] | Logged in as: $username , running $osinfo enviroment | $date"
$PSDefaultParameterValues['*:ForegroundColor'] = 'Red'
Clear-Host

### >>> Disable X button <<< ###
function Disable-X {
    #Calling user32.dll methods for Windows and Menus
    $MethodsCall = '
    [DllImport("user32.dll")] public static extern long GetSystemMenu(IntPtr hWnd, bool bRevert);
    [DllImport("user32.dll")] public static extern bool EnableMenuItem(long hMenuItem, long wIDEnableItem, long wEnable);
    [DllImport("user32.dll")] public static extern long SetWindowLongPtr(long hWnd, long nIndex, long dwNewLong);
    [DllImport("user32.dll")] public static extern bool EnableWindow(long hWnd, int bEnable);
    '
    #Create a new namespace for the Methods to be able to call them
    Add-Type -MemberDefinition $MethodsCall -name NativeMethods -namespace Win32

    $SC_CLOSE = 0xF060
    $MF_DISABLED = 0x00000002L


    #Create a new namespace for the Methods to be able to call them
    Add-Type -MemberDefinition $MethodsCall -name NativeMethods -namespace Win32

    $PSWindow = Get-Process -Pid $PID
    $hwnd = $PSWindow.MainWindowHandle

    #Get System menu of windows handled
    $hMenu = [Win32.NativeMethods]::GetSystemMenu($hwnd, 0)

    #Disable X Button
    [Win32.NativeMethods]::EnableMenuItem($hMenu, $SC_CLOSE, $MF_DISABLED) | Out-Null
}
Disable-X


Set-ItemProperty -Path 'HKLM:\SOFTWARE\Wow6432Node\Microsoft\.NetFramework\v4.0.30319' -Name 'SchUseStrongCrypto' -Value '1' -Type DWord # 
Set-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\.NetFramework\v4.0.30319' -Name 'SchUseStrongCrypto' -Value '1' -Type DWord # 
Clear-Host
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
ServicePointManager.SecurityProtocol = SecurityProtocolType.Tls12;

Write-Host "Preparing script. . ."
Start-Sleep 1
Clear-Host
Write-Host "Progress: [=======] 20% "
Write-Host "Creating a safe space for dependencies. . ."
New-Item "$PSScriptRoot/requirements" -itemType Directory -Force  | out-null
Start-Sleep 1
Clear-Host
Write-Host "Progress: [=================] 40%"
Write-Host "Getting prerequisites. . ."
winget install --id Microsoft.NuGet --source winget | out-null
Start-Sleep 2
# Invoke-WebRequest -Uri "direct_link" -OutFile "$PSScriptRoot\requirements\loader.mp3" | Out-Null # You can add multiple songs, so that the "skip song" etc work.
Install-PackageProvider -Name NuGet  -Force -Confirm:$false | out-null
Clear-Host
Write-Host "Progress: [========================================] 100%"
Write-Host "Starting prerequisites. . ."
Start-Sleep 1
Import-PackageProvider -Name NuGet -Force | out-null   
Import-Module NuGet | out-null
Install-Module -Name MusicPlayer -Force -Confirm:$false | out-null 
Import-Module MusicPlayer | out-null 
Clear-Host
Write-Host "Finalizing load. . ."
Start-Sleep 2
Play "$PSScriptRoot\requirements\" -Shuffle -Loop | Out-Null
Clear-Host
function Show-Menu
{
    param (
        [string]$Title = 'Privacy Enforcer v1.0'
    )
    Clear-Host
    Write-Host "+------------------+"
    Write-Host "| $Title | "
    Write-Host "+------------------+"
    Write-Host " "
    Write-Host ">>>> Blockers <<<<" -ForegroundColor White
    Write-Host "1: Press '1' to disallow usage of common ScreenShare tools"
    Write-Host "2: Press '1' to allow usage of common ScreenShare tools"
    Write-Host " "
    Write-Host ">>>> Cleaners <<<<" -ForegroundColor White
    Write-Host "3: Press '3' to open File Manager" #everything, delete files etcetc
    Write-Host "4: Press '4' to clear LastActivity logs (beta)"
    Write-Host "5: Press '5' to open Manual IP Blocker"
    Write-Host "6: Press '6' to clear browser history/cache [Chrome, Opera, Firefox]"
    Write-Host " "
    Write-Host ">>>> Extras <<<<" -ForegroundColor White
    Write-Host "H: Press 'H' for help"
    Write-Host "C: Press 'C' for credits"
    Write-Host "M: Press 'M' for music control"
    Write-Host "Q: Press 'Q' to exit"
    Write-Host " "
}

Clear-Host

do
 {
     Show-Menu
     $selection = Read-Host "Odaberite opciju"
     switch ($selection)
     {
         '1' {  
                New-ItemProperty -path "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\explorer\" -Name "DisallowRun" -PropertyType DWord -Value "1" | out-null
                New-item -Path "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\explorer\" -Name "DisallowRun" --no-verbose | out-null
                $1 = "SafeShareApplication.exe"
                New-ItemProperty -path "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\explorer\DisallowRun" -Name "2" -PropertyType String -Value "$1" | out-null
                    Set-Service -Name mpssvc -StartupType 'Automatic' | out-null
                    Start-Service mpssvc | out-null
                    New-NetFirewallRule -DisplayName "SafeShareBlock" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 172.67.140.195#| out-null
                    New-NetFirewallRule -DisplayName "SafeShareBlock" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 172.67.140.195  | out-null
                    New-NetFirewallRule -DisplayName "SafeShareBlock2" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 104.21.38.231 | out-null 
                    New-NetFirewallRule -DisplayName "SafeShareBlock2" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 104.21.38.231 | out-null
                    New-NetFirewallRule -DisplayName "SafeShareBlock3" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 188.114.97.12 | out-null 
                    New-NetFirewallRule -DisplayName "SafeShareBlock3" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 188.114.97.12 | out-null # 188.114.96.12
					New-NetFirewallRule -DisplayName "SafeShareBlock4" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 151.101.193.69 | out-null 
                    New-NetFirewallRule -DisplayName "SafeShareBlock4" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 151.101.193.69 | out-null 
                    New-NetFirewallRule -DisplayName "SafeShareBlock5" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 2a06:98c1:3121::c
                    New-NetFirewallRule -DisplayName "SafeShareBlock5" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 2a06:98c1:3121::c | out-null # safeshare.solutions
                    New-NetFirewallRule -DisplayName "SafeShareBlock5" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 188.114.96.12 | out-null
                    New-NetFirewallRule -DisplayName "SafeShareBlock5" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 188.114.96.12 | out-null # 162.211.80.236
                    New-NetFirewallRule -DisplayName "VoidTools1" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 162.211.80.236 | out-null 
                    New-NetFirewallRule -DisplayName "VoidTools1" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 162.211.80.236 | out-null 
                    New-NetFirewallRule -DisplayName "NirSoft1" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 138.128.181.29 | out-null
                    New-NetFirewallRule -DisplayName "NirSoft1" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress 138.128.181.29 | out-null
                    Add-Content -Path $env:windir\System32\drivers\etc\hosts -Value "`n127.0.0.1 safeshare.solutions`n127.0.0.2 voidtools.com" -Force | out-null 
                    Add-Content -Path $env:windir\System32\drivers\etc\hosts -Value "`n127.0.0.2 voidtools.com" -Force | out-null 
                    Add-Content -Path $env:windir\System32\drivers\etc\hosts -Value "`n127.0.0.3 nirsoft.net" -Force | out-null
                    Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache\*" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 
                    Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache2\entries\*" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 
                    Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cookies" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 
                    Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Media Cache" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 
                    Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cookies-Journal" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 

                    Stop-Service mpssvc | out-null 
                        Start-Sleep 1
                        Start-Service  mpssvc | out-null 

                    $registryPath = "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
                    If ( !(Test-Path $registryPath) ) { New-Item -Path $registryPath -Force; };
                    New-ItemProperty -Path $registryPath -Name "Start_TrackDocs" -Value 0 -PropertyType DWORD -Force | out-null
    
                    $registryPath2 = "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
                    If ( !(Test-Path $registryPath2) ) { New-Item -Path $registryPath2 -Force; };
                    New-ItemProperty -Path $registryPath2 -Name "Start_TrackProgs" -Value 0 -PropertyType DWORD -Force | out-null

                Clear-Host
             'Blokiranje zavrseno.'
         } '2' {
             Remove-Item -Path HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\explorer\DisallowRun -Force | out-null
             Remove-NetFirewallRule -DisplayName "SafeShareBlock"  | out-null
             Remove-NetFirewallRule -DisplayName "SafeShareBlock2" | out-null
             Remove-NetFirewallRule -DisplayName "SafeShareBlock3" | out-null
			 Remove-NetFirewallRule -DisplayName "SafeShareBlock4" | out-null
             Remove-NetFirewallRule -DisplayName "SafeShareBlock5" | out-null
             Remove-NetFirewallRule -DisplayName "VoidTools1" | out-null
             Remove-NetFirewallRule -DisplayName "VoidTools1" | out-null
             Remove-NetFirewallRule -DisplayName "NirSoft1" | Out-Null
             (get-content $env:windir\System32\drivers\etc\hosts) -replace "127.0.0.1 safeshare.solutions","" |  Out-File $env:windir\System32\drivers\etc\hosts | out-null
             (get-content $env:windir\System32\drivers\etc\hosts) -replace "127.0.0.2 voidtools.com","" |  Out-File $env:windir\System32\drivers\etc\hosts | out-null
             (get-content $env:windir\System32\drivers\etc\hosts) -replace "127.0.0.3 nirsoft.net","" |  Out-File $env:windir\System32\drivers\etc\hosts | out-null

             $registryPath3 = "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
             If ( !(Test-Path $registryPath3) ) { New-Item -Path $registryPath3 -Force; };
             New-ItemProperty -Path $registryPath3 -Name "Start_TrackDocs" -Value 1 -PropertyType DWORD -Force | out-null
    
             $registryPath4 = "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
             If ( !(Test-Path $registryPath4) ) { New-Item -Path $registryPath4 -Force; };
             New-ItemProperty -Path $registryPath4 -Name "Start_TrackProgs" -Value 1 -PropertyType DWORD -Force | out-null

             Stop-Service mpssvc | out-null 
                        Start-Sleep 1
                        Start-Service  mpssvc | out-null 

             Clear-Host
             'Odblokiranje zavrseno'
         } '3' {
              do {
                Clear-Host
                Write-Host "File Manager (in development)"
                Write-Host "1: Search for a file"
                Write-Host "2: Delete a file"
                Write-Host "3: Delete all .log files"
                Write-Host "4: Folder Locker"
                Write-Host "B: To the previous menu"
                $internal3 = Read-Host "Choose an option"
                switch ($internal3)
                 {
                      '1' {
                        Clear-Host
                        $itemdr = Read-Host ">> Type drive letter" 
                        $itemsr = Read-Host ">> Write file name" 
                        $conf1 = Read-Host "You chose to search on the '$itemdr' drive for a file that contains '$itemsr' in its name, continue? [Y/N]" 
                        if ($conf1 -eq 'Y') {
                                $itemdrC = $itemdr.replace(':','') # Quality of Life function
                                Start-Process cmd -verb runas -ArgumentList "/k @echo off && title FindFile && dir ${itemdrC}:\*$itemsr* /n /s /p /b && cd/"
                        } else {
                            Write-Host "Aborting... Returning to the previous menu."
                            Start-Sleep 1
                        }                                      
                        
                        #Start-Process cmd -verb runas -ArgumentList "/k title FindFile && dir C:\*$itemsr* /n /s /p /b  && dir D:\*$itemsr* /n /s /p /b && dir E:\*$itemsr* /n /s /p /b && dir F:\*$itemsr* /n /s /p /b && dir G:\*$itemsr* /n /s /p /b && dir H:\*$itemsr* /n /s /p /b && dir A:\*$itemsr* /n /s /p /b && dir X:\*$itemsr* /n /s /p /b && dir L:\*$itemsr* /n /s /p /b"                  
                    } '2' {
                        Clear-Host
                        $itemde = Read-Host ">> Write file path"
                        Write-Host " "
                        $conf2 = Read-Host "You chose to delete a file '$itemde', continue? [Y/N]"
                        if ($conf2 -eq 'Y') {
                            Start-Process cmd -verb runas -ArgumentList "/k @echo off && title DeleteFile && del $itemde /f && echo File Deleted: '$itemde' && TIMEOUT 5 && exit "
                        } else {
                            Write-Host "Aborting... Returning to the previous menu."
                            Start-Sleep 1
                        }
                        
                    } '3' {
                        Clear-Host
                        Write-Host "This will clean all .log files on your system root, can make some programs unstable."
                        $conf3 = Read-Host "Do you want to continue? [Y/N]"
                        if ($conf3 -eq 'Y') {
                            Start-Process cmd -verb runas -ArgumentList "/k @echo off && cd/ && del *.log /a /s /q /f && cls && echo All 'Log Files Removed Successfully!' "
                        } else {
                            Write-Host "Aborting... Returning to the previous menu."
                            Start-Sleep 1
                        }
                        
                    } '4' {
                        
                        
                        do {
                        Clear-Host
                        Write-Host "Folder Locker: Makes folders inaccessible to view [useful for locking private data]"
                        Write-Host " "
                        Write-Host "1. Lock a folder"
                        Write-Host "2. Unlock a folder"
                        Write-Host "3. 3rd-party Locker Download"
                        Write-Host "B: To the previous menu"
                        $conf4 = Read-Host "Choose an option"
                        switch($conf4) {
                                '1' {
                                    Clear-Host
                                    $lockpath = Read-Host "Input folder path to lock"
                                    Write-Host "Locking folder '$lockpath'"
                                    cacls $lockpath /P everyone:n
                                }
                                '2' {
                                    Clear-Host
                                    $unlockpath = Read-Host "Input folder path to unlock"
                                    Write-Host "Unlocking folder '$unlockpath'"
                                    cacls $unlockpath /P everyone:f
                                }
                            }


                        } until ($conf4 -eq 'b')
                    }

              } } until ($internal3 -eq 'b')
         
             
             
             
         } '4' {
              Clear-Host
              Invoke-WebRequest -Uri "https://app.box.com/index.php?rm=box_download_shared_file&shared_name=i3bpxb3iboiax66b1p83nmkkp9qtax7n&file_id=f_1081249714243" -OutFile "$PSScriptRoot\requirements\clear_trash.bat" | Out-Null
              Clear-Host
              Write-Host "Starting cleaner . . ."
              Start-Sleep 2
              Invoke-Item "$PSScriptRoot\requirements\clear_trash.bat"
              Write-Host "Cleaning Finished"
              Start-Sleep 2 
              Remove-Item "$PSScriptRoot\requirements\clear_trash.bat" -Recurse -Force -Confirm:$false | Out-Null

              
         } '5' {
            do {
            Clear-Host
            Write-Host "Manual IP blocker (under development)"
            Write-Host "1: Resolve IP<=>Hostname"
            Write-Host "2: Block IP"
            Write-Host "3: Unblock IP"
            Write-Host "4: List all Firewall rules"
            Write-Host "5: Block Hostname"
            Write-Host "6: Unblock Hostname"
            Write-Host "7: List blocked hostnames"
            Write-Host "H: Help"
            Write-Host "B: To the previous menu"
            $internal5 = Read-Host "Choose an option"
                switch ($internal5) {
                    '1' {
                        Clear-Host
                        Write-Host "Resolve IP/Hostname"
                        $resolver = Read-Host "Input Hostname or IP to resolve"
                        Write-Host "Resolving results:"
                        Resolve-DnsName -name $resolver -NoHostsFile
                        Write-Host -NoNewLine 'Press any key to continue...';
                        $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');
                   }'2' {
                        Clear-Host
                        Write-Host "Block IP"
                        Set-Service -Name mpssvc -StartupType 'Automatic' | out-null
                        Start-Service mpssvc | out-null
                        $var21 = Read-Host "Set Display Name"
                        $var22 = Read-Host "Input IP Address"
                        New-NetFirewallRule -DisplayName "$var21" -Direction Inbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress $var22 | Out-Null
                        New-NetFirewallRule -DisplayName "$var21" -Direction Outbound -LocalPort Any -Protocol Any -Action Block -RemoteAddress $var22 | Out-Null
                        Write-Host "Blocked IP $var22, displayed as $var21 in Firewall settings."
                        Write-Host " "
                        Stop-Service mpssvc | out-null 
                        Start-Sleep 1
                        Start-Service  mpssvc | out-null 
                        Write-Host -NoNewLine 'Press any key to continue...';
                        $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');

                   }'3' {
                        Clear-Host
                        Start-Service mpssvc | out-null 
                        Write-Host "Unblock IP"
                        $var31 = Read-Host "Input IP Address"
                        Remove-NetFirewallRule -DisplayName "$var31"  | out-null
                        Write-Host "Removed block of an IP under Display Name $var31"
                        Write-Host " "
                        Stop-Service mpssvc | out-null 
                        Start-Sleep 1
                        Start-Service  mpssvc | out-null 
                        Write-Host -NoNewLine 'Press any key to continue...';
                        $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');
                   } '4' {
                        #List Firewall 
                        Clear-Host
                        Get-NetFirewallRule | Sort-Object -Descending |
                        Format-Table -Property DisplayName,
                        @{Name='Protocol';Expression={($PSItem | Get-NetFirewallPortFilter).Protocol}},
                        @{Name='LocalPort';Expression={($PSItem | Get-NetFirewallPortFilter).LocalPort}},
                        @{Name='RemotePort';Expression={($PSItem | Get-NetFirewallPortFilter).RemotePort}},
                        @{Name='RemoteAddress';Expression={($PSItem | Get-NetFirewallAddressFilter).RemoteAddress}},
                        Enabled,
                        Profile,
                        Direction,
                         Action

                         Write-Host " "
                         Write-Host -NoNewLine 'Press any key to continue...';
                         $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');
                   } '5' {
                    Clear-Host
                    $hostb = Read-Host "Input hostname you want to block"
                    if ($hostb -like "*https://*") {$itemdrA = $hostb.replace('https://',''); Write-Host "Blocked $itemdrA"; Add-Content -Path $env:windir\System32\drivers\etc\hosts -Value "`n127.0.0.1 $itemdrA" -Force | out-null ;}
                    elseif ($hostb -like "*http://*") {$itemdrB = $hostb.replace('http://',''); Write-Host "Blocked $itemdrB"; Add-Content -Path $env:windir\System32\drivers\etc\hosts -Value "`n127.0.0.1 $itemdrB" -Force | out-null ;} 
                    
                    Write-Host "Blocked $itemdrA"
                    Write-Host " "
                    Write-Host -NoNewLine 'Press any key to continue...';
                    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');
                    Clear-Host
                   } '6' {
                    Clear-Host
                    $hostc = Read-Host "Input hostname you want to unblock"
                    $hostsc = (get-content $env:windir\System32\drivers\etc\hosts)
                    if ($hostsc -like "127.0.0.1 *$hostc*") { (get-content $env:windir\System32\drivers\etc\hosts) -replace "127.0.0.1 *$hostc*","" |  Out-File $env:windir\System32\drivers\etc\hosts | out-null ;}
                    (get-content $env:windir\System32\drivers\etc\hosts) -replace "127.0.0.1 *$hostc*","" |  Out-File $env:windir\System32\drivers\etc\hosts | out-null
                    Write-Host -NoNewLine 'Press any key to continue...';
                    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');
                    Clear-Host 
                   
                   } '7' {
                    Clear-Host
                    more "$env:windir\System32\drivers\etc\hosts"
                    Write-Host -NoNewLine 'Press any key to continue...';
                    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');
                    Clear-Host 

                   } 'H' {
                    Clear-Host
                    Write-Host "This tool is using firewall to block communication to certain IPs... can be used as an Parental Control tool."
                    Write-Host "Usage of IP Blocker:"
                    Write-Host "Use 1st option to get the IP Adress of a Site you want to block (exp. www.google.com)"
                    Write-Host "Write under which name the Rule will be displayed in Firewall rule [Set Display Name]"
                    Write-Host "Insert that IP when using the 2nd option"
                    Write-Host "You can use the 4th option to list all firewall rules"
                    Write-Host " "
                    Write-Host "Usage of Hostname Blocker:"
                    Write-Host "Use the 1st option if you have the IP but not the Hostname"
                    Write-Host "Write the Hostname [ex. www.google.com] and voila its gonna be added to the blocklist"
                    Write-Host "You can use the 7th option to list 'hosts' file."
                    Write-Host " "
                    Write-Host -NoNewLine 'Press any key to continue...';
                    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown');
                  }

                }

            } until ($internal5 -eq 'b') 
            
            
         } '6' {
             Clear-Host   
             #Chrome Clean Method
             $Items = @('Archived History',
            'Cache\*',
            'Cookies',
            'History',
            'Login Data',
            'Top Sites',
            'Visited Links',
            'Web Data')
                $Folder = "$($env:LOCALAPPDATA)\Google\Chrome\User Data\Default"
                $Items |  ForEach-Object {         # was %
              if (Test-Path "$Folder\$_") {
                  Remove-Item "$Folder\$_"   -Recurse -Force -EA SilentlyContinue | out-null
                   }
               }
              Remove-Item "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\History" -Recurse -Force -EA SilentlyContinue | out-null
              Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache\*" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 
              Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cache2\entries\*" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 
              Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cookies" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 
              Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Media Cache" -Recurse -Force -EA SilentlyContinue -Verbose | out-null 
              Remove-Item -path "$env:LOCALAPPDATA\Google\Chrome\User Data\Default\Cookies-Journal" -Recurse -Force -EA SilentlyContinue -Verbose | out-null
              Clear-Host
              Write-Host "Removing Google Chrome data . . . "
              Start-Sleep 5
              #Mozilla ForceRemove
              Remove-Item "$env:APPDATA\Mozilla\Firefox\Profiles\" -Recurse -Force -EA SilentlyContinue | out-null
              Remove-Item "$env:LOCALAPPDATA\Mozilla\Firefox\Profiles\" -Recurse -Force -EA SilentlyContinue | out-null
              Clear-Host
              Write-Host "Removing Google Chrome data . . ."
              Write-Host "Removing Mozilla Firefox data . . . "
              Start-Sleep 4
             #Opera ForceRemove
              Remove-Item "$env:APPDATA\Opera Software" -Recurse -Force -EA SilentlyContinue | out-null
              Clear-Host
              Write-Host "Removing Google Chrome data . . ."
              Write-Host "Removing Mozilla Firefox data . . ."
              Write-Host "Removing Opera data . . ."
              Start-Sleep 5
              Write-Host
              Write-Host "DONE"
              Start-Sleep 3

              
         } 'H' {
              Clear-Host
              Write-Host "  ______   ______     ______  "
              Write-Host " /\  == \ /\  ___\   /\  ___\ "
              Write-Host " \ \  _-/ \ \  __\   \ \  __\ "
              Write-Host "  \ \_\    \ \_____\  \ \_\   " 
              Write-Host "   \/_/     \/_____/   \/_/   "
              Write-Host "------------------------------"
              Write-Host "Privacy Enforcer"
              Write-Host " "
              Write-Host "Help:" -ForegroundColor White
              Write-Host ""
              Write-Host "This section will briefly explain to you what certain options do"
              Write-Host "" 
              Write-Host "1-2 Disables accessing commonly known applications used in ScreenShares by blocking their IP and disables windows' ran exes tracking options"
              Write-Host "3 Search for a file, delete it without logs and also delete all .log files on your drive"
              Write-Host "4 Removes different logging techniques windows uses of ran executeables on the system"
              Write-Host "5 Manually block IP/Hosts [additional help in the sub-menu]"
              Write-Host "6 Clears cookies and history of Chrome, Opera and Firefox"

         } 'C' {
              Clear-Host
              Write-Host "  ______   ______     ______  "
              Write-Host " /\  == \ /\  ___\   /\  ___\ "
              Write-Host " \ \  _-/ \ \  __\   \ \  __\ "
              Write-Host "  \ \_\    \ \_____\  \ \_\   " 
              Write-Host "   \/_/     \/_____/   \/_/   "
              Write-Host "------------------------------"
              Write-Host "Privacy Enforcer"
              Write-Host " "
              Write-Host "Credits:" -ForegroundColor White
              Write-Host "github.com/Master0fFate"
              Write-Host "github.com/Baseult"
              Write-Host " "
              Write-Host "Music:" -ForegroundColor White
              Write-Host "youtube.com/@LHSchiptunes"
              Write-Host " "
              Write-Host "This script has been used as a learning project for Shell lang," 
              Write-Host "it might be absolutely useless, but still it served it's purpose as now I'm familliar with Shell"
              Write-Host " "
              Write-Host "Release year: 2023"


         } 'M' {
            Clear-Host
            Write-Host "Music Muted"
            do
                { 
                Clear-Host
                Write-Host "Music section, 'B' to go back."
                Write-Host " "
                Write-Host "WARN: This works only if you modified the script"
                Write-Host ""
                Write-Host "1: Press '1' to Mute music"
                Write-Host "2: Press '2' to Skip current song"
                Write-Host "3: Press '3' to Play music"
                Write-Host "B: To go to the previous menu."
                Write-Host " "
                $internalM = Read-Host "Choose an option"
                switch ($internalM)
                 {
                      '1' { Play -Stop |Out-Null
                    } '2' { Play -Stop |Out-Null; Play "$PSScriptRoot\requirements\" -Shuffle -Loop | Out-Null
                    } '3' { Play "$PSScriptRoot\requirements\" -Shuffle -Loop | Out-Null 
                    }
                }
              } until ($internalM -eq 'b')            
         }
     }
     pause
 }
 until ($selection -eq 'q')
 Remove-Item "$PSScriptRoot\requirements" -Recurse -Force -Confirm:$false | Out-Null
 stop-process -Id $PID

 

#New-ItemProperty -path "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\explorer\" -Name "DisallowRun" -PropertyType DWord -Value "1"
#New-item -Path "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\explorer\" -Name "DisallowRun"
#$1 = "test.exe"dawd
#New-ItemProperty -path "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\explorer\DisallowRun" -Name "2" -PropertyType String -Value "$1"
#Remove-Item -Path HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\explorer\DisallowRun -Force -Verbose\
#$debug11 = Get-BcdEntryDebugSettings
