@echo off
net session >nul 2>&1
    if %errorLevel% == 0 (
        goto admin
    ) else (
        goto NOADMIN
    )
:admin
echo.
@echo off & setlocal
color 4
echo msgbox"Do not exit the script randomly, use the right procedures.">%temp%\msg.vbs
call %temp%\msg.vbs
del %temp%\msg.vbs
echo.
goto NEXT

:NOADMIN
echo.
@echo off & setlocal
color c
echo msgbox"Launch as Admin">%temp%\noadmin.vbs
call %temp%\noadmin.vbs
del %temp%\noadmin.vbs
exit

:NEXT
cd %temp%
if not exist "%temp%\PEF" mkdir "%temp%\PEF"
echo "Preparing autorun. . ."
timeout 2
echo.
echo Downloading prerequisites
echo.
echo.
curl -L "https://raw.githubusercontent.com/Master0fFate/PrivacyEnforcerPS/main/pef_devver.ps1" -o "%temp%/PEF/pefxd.ps1"

curl -L "https://github.com/PowerShell/PowerShell/releases/download/v7.3.4/PowerShell-7.3.4-win-x64.msi" -o "%temp%/PEF/PowerShell-7.3.0-win-x64.msi"
cd %temp%/PEF

echo Download complete, starting install

msiexec.exe /package PowerShell-7.3.4-win-x64.msi /quiet ADD_EXPLORER_CONTEXT_MENU_OPENPOWERSHELL=1 ADD_FILE_CONTEXT_MENU_RUNPOWERSHELL=1 ENABLE_PSREMOTING=1 REGISTER_MANIFEST=1 USE_MU=1 ENABLE_MU=1 ADD_PATH=1 

cd %temp%
cls
title Privacy Enforcer Autorun pre-release_debug3
color 4
echo "  _____      _                            "
echo " |  __ \    (_)                           "
echo " | |__) | __ ___   ____ _  ___ _   _      "
echo " |  ___/ '__| \ \ / / _` |/ __| | | |     "
echo " | |   | |  | |\ V / (_| | (__| |_| |     "
echo " |_|___|_|  |_| \_/ \__,_|\___|\__, |     "
echo " |  ____|      / _|             __/ |     "
echo " | |__   _ __ | |_ ___  _ __ __|___/ _ __ "
echo " |  __| | '_ \|  _/ _ \| '__/ __/ _ \ '__|"
echo " | |____| | | | || (_) | | | (_|  __/ |   "
echo " |______|_| |_|_| \___/|_|  \___\___|_|   "
echo.

Set "STRING=Starting Privacy Enforcer . . ."

For /F %%A In ('"Prompt $H&For %%B In (1) Do Rem"') Do Set "BS=%%A"

For /F Delims^=^ EOL^= %%A In ('"(CMD/U/CEcho=%STRING%)|Find /V """'
) Do Set/P "=a%BS%%%A"<Nul & PathPing 127.0.0.1 -n -q 1 -p 100 1>Nul
timeout 10
cls

pwsh --version >nul 2>&1 && ( pwsh "%temp%/PEF/pefxd.ps1" ) || ( powershell "%temp%/PEF/pefxd.ps1" )

cd %temp%

if exist "%temp%/PEF" rmdir /s /q "%temp%/PEF"
echo "FINISHED"
timeout 2
exit
