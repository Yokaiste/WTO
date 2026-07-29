@echo off
setlocal EnableExtensions DisableDelayedExpansion

:: ============================================================
:: CONFIG - the only section that differs between mods.
:: Copy this installer into another mod and edit only this block.
:: ============================================================
set "DEPLOY_TITLE=WTO + YSM Auto Deploy"
set "DEPLOY_SUBJECT=WTO and YSM"
set "DEPLOY_MOD_ARGS=--mod ysm --mod wto"
set "DEPLOY_TEMP_PREFIX=wto"
set "DEPLOY_RELEASE_REPO=WTO"
set "DEPLOY_REPO_OWNER=Yokaiste"
set "DEPLOY_REPO_COUNT=2"
goto start

:deploy_repositories
call :deploy_repository "YSM"
if errorlevel 1 exit /b 1
call :deploy_repository "WTO"
if errorlevel 1 exit /b 1
exit /b 0

:: ============================================================
::  Shared Auto Deploy logic. Everything below this line is
::  identical in every mod that uses this installer.
:: ============================================================

:start
:: %0 becomes the label name inside `call :label`, so the file name is captured
:: here, in the top-level context, and used everywhere below.
set "DEPLOY_SELF=%~nx0"
set "MOD_ROOT=%~dp0"
if "%MOD_ROOT:~-1%"=="\" set "MOD_ROOT=%MOD_ROOT:~0,-1%"
set "YMB_DIR=%MOD_ROOT%\YMB"
set "YMB_REPOSITORY=https://github.com/%DEPLOY_REPO_OWNER%/YMB"
:: Every mod publishes its configuration package under the same tag and asset
:: name, so each one resolves to a fixed URL with no release lookup.
set "DEPLOY_RELEASE_TAG=stable"
set "DEPLOY_ARCHIVE_SUFFIX=-config.zip"
set /a DEPLOY_STEP=0
set /a DEPLOY_TOTAL_STEPS=3+DEPLOY_REPO_COUNT

if /I "%~1"=="--help" goto help
if /I "%~1"=="/?" goto help

echo.
echo ============================================================
echo   %DEPLOY_TITLE%
echo ============================================================
echo.

call :validate_mod_root
if errorlevel 1 goto failed

call :check_tools
if errorlevel 1 goto failed

call :create_temp
if errorlevel 1 goto failed

call :download_ymb
if errorlevel 1 goto failed

call :deploy_ymb
if errorlevel 1 goto failed

call :deploy_repositories
if errorlevel 1 goto failed

call :build_preview
if errorlevel 1 goto failed

call :cleanup
echo.
echo ============================================================
echo   %DEPLOY_SUBJECT% is ready to sync
echo ============================================================
echo.
echo YMB built and validated a preview. It did not change live WARNO files.
echo Review the preview under:
echo   "%YMB_DIR%\.ymb-build\output"
echo.
echo To apply %DEPLOY_SUBJECT%, run this from any terminal:
echo   "%YMB_DIR%\YMB.bat" sync %DEPLOY_MOD_ARGS% --yes
echo.
echo To restore the files saved by YMB later:
echo   "%YMB_DIR%\YMB.bat" recover %DEPLOY_MOD_ARGS% --yes
echo.
echo You do not need this deployment file for those commands. You can also
echo double-click "%YMB_DIR%\YMB.bat" and enter the shorter commands:
echo   sync %DEPLOY_MOD_ARGS% --yes
echo   recover %DEPLOY_MOD_ARGS% --yes
echo.
if not defined CI pause
endlocal & exit /b 0

:step
set /a DEPLOY_STEP+=1
echo [%DEPLOY_STEP%/%DEPLOY_TOTAL_STEPS%] %~1
exit /b 0

:validate_mod_root
for %%I in ("%MOD_ROOT%\..") do set "MODS_DIR=%%~fI"
for %%I in ("%MODS_DIR%") do set "PARENT_NAME=%%~nxI"

if /I not "%PARENT_NAME%"=="Mods" goto invalid_mod_root
if not exist "%MOD_ROOT%\CommonData\" goto invalid_mod_root
if not exist "%MOD_ROOT%\GameData\" goto invalid_mod_root
if exist "%YMB_DIR%\.git\" (
  echo [ERROR] "%YMB_DIR%" is a source Git checkout, not a portable YMB installation.
  echo Move or rename it before using Auto Deploy. No source files were overwritten.
  exit /b 1
)

echo [OK] WARNO mod root: "%MOD_ROOT%"
exit /b 0

:invalid_mod_root
echo [ERROR] This file is not inside a generated WARNO mod folder.
echo.
echo Create one with WARNO's included tools:
echo   1. In Steam, open WARNO ^> Properties ^> Installed Files ^> Browse.
echo   2. Open the Mods folder.
echo   3. Run: CreateNewMod.bat YourModName
echo   4. Copy %DEPLOY_SELF% into the new folder and run it there.
echo.
echo Expected location:
echo   ^<SteamLibrary^>\steamapps\common\WARNO\Mods\YourModName\%DEPLOY_SELF%
echo.
echo The same folder must contain both CommonData and GameData.
exit /b 1

:check_tools
where powershell.exe >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Windows PowerShell was not found on PATH.
  exit /b 1
)

:: Windows ships its own tar and robocopy, and they are resolved by full path on
:: purpose. Git for Windows, MSYS, and Cygwin all put a Unix `tar` on PATH that
:: reads "C:\..." as a remote host and fails, and PATH order is the user's, not
:: ours. PATH is only a fallback for a system without the System32 copies.
set "TAR_EXE=%SystemRoot%\System32\tar.exe"
if not exist "%TAR_EXE%" for %%T in (tar.exe) do set "TAR_EXE=%%~$PATH:T"
if not exist "%TAR_EXE%" (
  echo [ERROR] Windows tar.exe was not found.
  echo Install current Windows updates and retry.
  exit /b 1
)

set "ROBOCOPY_EXE=%SystemRoot%\System32\robocopy.exe"
if not exist "%ROBOCOPY_EXE%" for %%T in (robocopy.exe) do set "ROBOCOPY_EXE=%%~$PATH:T"
if not exist "%ROBOCOPY_EXE%" (
  echo [ERROR] Windows robocopy.exe was not found.
  exit /b 1
)

echo [OK] Required Windows tools found. Git is not needed.
exit /b 0

:create_temp
set "DEPLOY_TEMP=%TEMP%\%DEPLOY_TEMP_PREFIX%-auto-deploy-%RANDOM%-%RANDOM%"
set "YMB_URL_FILE=%DEPLOY_TEMP%\ymb-url.txt"
set "YMB_ARCHIVE=%DEPLOY_TEMP%\ymb.zip"
set "YMB_EXTRACT=%DEPLOY_TEMP%\release"

mkdir "%YMB_EXTRACT%" >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Could not create the temporary deployment directory:
  echo   "%DEPLOY_TEMP%"
  exit /b 1
)
exit /b 0

:: Downloads %~1 to %~2. One place for every download so the YMB release and
:: each mod package are fetched the same way.
:download_file
set "DOWNLOAD_URL=%~1"
set "DOWNLOAD_TARGET=%~2"
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue'; Invoke-WebRequest -UseBasicParsing -Uri $env:DOWNLOAD_URL -OutFile $env:DOWNLOAD_TARGET"
if errorlevel 1 exit /b 1
exit /b 0

:download_ymb
echo.
call :step "Finding the latest portable YMB release..."
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $headers=@{Accept='application/vnd.github+json'; 'User-Agent'='YMB-Auto-Deploy'}; $release=Invoke-RestMethod -UseBasicParsing -Headers $headers -Uri ('https://api.github.com/repos/' + $env:DEPLOY_REPO_OWNER + '/YMB/releases/latest'); $asset=$release.assets ^| Where-Object { $_.name -match '^YMB-v.+-windows-x64\.zip$' -and $_.name -notmatch '-no-bun\.zip$' } ^| Select-Object -First 1; if ($null -eq $asset) { throw 'The full Windows YMB archive is missing from the latest release.' }; [IO.File]::WriteAllText($env:YMB_URL_FILE, $asset.browser_download_url, [Text.UTF8Encoding]::new($false))"
if errorlevel 1 (
  echo [ERROR] Could not find the full YMB archive at %YMB_REPOSITORY%/releases/latest
  exit /b 1
)

set /p "YMB_URL="<"%YMB_URL_FILE%"
if not defined YMB_URL (
  echo [ERROR] The YMB release returned an empty download URL.
  exit /b 1
)

call :step "Downloading portable YMB..."
call :download_file "%YMB_URL%" "%YMB_ARCHIVE%"
if errorlevel 1 (
  echo [ERROR] YMB download failed: "%YMB_URL%"
  exit /b 1
)

"%TAR_EXE%" -xf "%YMB_ARCHIVE%" -C "%YMB_EXTRACT%"
if errorlevel 1 (
  echo [ERROR] The downloaded YMB archive could not be extracted.
  exit /b 1
)

if not exist "%YMB_EXTRACT%\YMB\YMB.bat" (
  echo [ERROR] The YMB release has an unexpected layout.
  exit /b 1
)
if not exist "%YMB_EXTRACT%\YMB\runtime\bun.exe" (
  echo [ERROR] The selected YMB release is missing its required runtime.
  exit /b 1
)
exit /b 0

:deploy_ymb
call :step "Deploying the production YMB package..."
if not exist "%YMB_DIR%\" goto copy_ymb
if exist "%YMB_DIR%\.git\" (
  echo [ERROR] "%YMB_DIR%" is a source Git checkout, not a portable YMB installation.
  echo Move or rename it before using Auto Deploy. No source files were overwritten.
  exit /b 1
)
if exist "%YMB_DIR%\YMB.bat" goto clean_ymb
if exist "%YMB_DIR%\package.json" goto clean_ymb

echo [ERROR] "%YMB_DIR%" already exists but is not a recognized YMB installation.
echo Move or rename that folder, then run this file again.
exit /b 1

:clean_ymb
for %%D in (app docs runtime types) do if exist "%YMB_DIR%\%%D\" rmdir /s /q "%YMB_DIR%\%%D"

:copy_ymb
"%ROBOCOPY_EXE%" "%YMB_EXTRACT%\YMB" "%YMB_DIR%" /E /COPY:DAT /DCOPY:DAT /R:2 /W:1 /NFL /NDL /NJH /NJS /NP >nul
set "ROBOCOPY_EXIT=%ERRORLEVEL%"
if %ROBOCOPY_EXIT% GEQ 8 (
  echo [ERROR] YMB deployment failed with robocopy exit code %ROBOCOPY_EXIT%.
  exit /b 1
)

if not exist "%YMB_DIR%\YMB.bat" (
  echo [ERROR] YMB.bat is missing after deployment.
  exit /b 1
)
if not exist "%YMB_DIR%\runtime\bun.exe" (
  echo [ERROR] The YMB runtime is missing after deployment.
  exit /b 1
)

"%YMB_DIR%\runtime\bun.exe" --version >"%DEPLOY_TEMP%\bun-version.txt"
if errorlevel 1 (
  echo [ERROR] The deployed YMB runtime could not start.
  exit /b 1
)
set /p "BUN_VERSION="<"%DEPLOY_TEMP%\bun-version.txt"
echo [OK] YMB runtime %BUN_VERSION%
exit /b 0

:: Installs one mod from its published configuration package: the `config`
:: folder YMB builds from, plus that repository's legal text. No Git, and no
:: full source tree on disk.
:deploy_repository
set "REPO_NAME=%~1"
set "REPO_DIR=%YMB_DIR%\mods\%~1"
set "REPO_ARCHIVE=%DEPLOY_TEMP%\%~1%DEPLOY_ARCHIVE_SUFFIX%"
set "REPO_EXTRACT=%DEPLOY_TEMP%\%~1-package"
set "REPO_STAGED=%REPO_EXTRACT%\%~1"
set "REPO_URL=https://github.com/%DEPLOY_REPO_OWNER%/%~1/releases/download/%DEPLOY_RELEASE_TAG%/%~1%DEPLOY_ARCHIVE_SUFFIX%"

call :step "Downloading and installing %REPO_NAME%..."
mkdir "%REPO_EXTRACT%" >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Could not create the temporary directory for %REPO_NAME%.
  exit /b 1
)

call :download_file "%REPO_URL%" "%REPO_ARCHIVE%"
if errorlevel 1 (
  echo [ERROR] Could not download the %REPO_NAME% configuration package:
  echo   %REPO_URL%
  echo Check your internet connection, then run %DEPLOY_SELF% again.
  exit /b 1
)

"%TAR_EXE%" -xf "%REPO_ARCHIVE%" -C "%REPO_EXTRACT%"
if errorlevel 1 (
  echo [ERROR] The downloaded %REPO_NAME% package could not be extracted.
  exit /b 1
)

if not exist "%REPO_STAGED%\config\ymb.mod.yaml" (
  echo [ERROR] The %REPO_NAME% package has an unexpected layout.
  echo Expected "%REPO_NAME%\config\ymb.mod.yaml" inside the archive.
  exit /b 1
)

if not exist "%REPO_DIR%\" goto copy_repository
if exist "%REPO_DIR%\.git\" (
  echo [ERROR] "%REPO_DIR%" is a source Git checkout, not a deployed package.
  echo Auto Deploy will not overwrite source you may have edited. Move or rename
  echo that folder, then run %DEPLOY_SELF% again. Nothing was changed.
  exit /b 1
)
if exist "%REPO_DIR%\config\ymb.mod.yaml" goto clean_repository

echo [ERROR] "%REPO_DIR%" already exists but is not a recognized %REPO_NAME%
echo installation. Move or rename that folder, then run this file again.
exit /b 1

:clean_repository
:: The package is the whole truth for `config`, so the old one is removed rather
:: than merged over. A file dropped from the mod must not survive an update.
rmdir /s /q "%REPO_DIR%\config"
if exist "%REPO_DIR%\config\" (
  echo [ERROR] Could not replace the existing configuration in:
  echo   "%REPO_DIR%\config"
  echo Close anything using those files, then run %DEPLOY_SELF% again.
  exit /b 1
)

:copy_repository
"%ROBOCOPY_EXE%" "%REPO_STAGED%" "%REPO_DIR%" /E /COPY:DAT /DCOPY:DAT /R:2 /W:1 /NFL /NDL /NJH /NJS /NP >nul
set "ROBOCOPY_EXIT=%ERRORLEVEL%"
if %ROBOCOPY_EXIT% GEQ 8 (
  echo [ERROR] %REPO_NAME% deployment failed with robocopy exit code %ROBOCOPY_EXIT%.
  exit /b 1
)

if not exist "%REPO_DIR%\config\ymb.mod.yaml" (
  echo [ERROR] %REPO_NAME% is missing its configuration after deployment.
  exit /b 1
)

echo [OK] %REPO_NAME% installed in "%REPO_DIR%"
exit /b 0

:build_preview
echo.
echo Validating the installed tools and %DEPLOY_SUBJECT% configuration...
call "%YMB_DIR%\YMB.bat" doctor
if errorlevel 1 exit /b 1

call "%YMB_DIR%\YMB.bat" validate %DEPLOY_MOD_ARGS%
if errorlevel 1 exit /b 1

echo.
echo Building a safe preview. Live WARNO files will not be changed...
call "%YMB_DIR%\YMB.bat" build %DEPLOY_MOD_ARGS%
if errorlevel 1 exit /b 1
exit /b 0

:cleanup
if defined DEPLOY_TEMP if exist "%DEPLOY_TEMP%\" rmdir /s /q "%DEPLOY_TEMP%"
exit /b 0

:failed
call :cleanup
echo.
echo Deployment stopped safely. No sync or recovery command was run.
echo Fix the error above, then run %DEPLOY_SELF% again.
echo.
if not defined CI pause
endlocal & exit /b 1

:help
echo %DEPLOY_TITLE%
echo.
echo Place this file inside a generated WARNO mod folder, beside its
echo CommonData and GameData folders, then double-click it.
echo.
echo It checks the required Windows tools, downloads the latest portable YMB
echo release and the published configuration package for %DEPLOY_SUBJECT%,
echo validates the configuration, and builds a preview. It never syncs or
echo recovers live WARNO files automatically.
echo.
echo Git is not required. A package holds only the configuration YMB builds
echo from. The full source lives at:
echo   https://github.com/%DEPLOY_REPO_OWNER%/%DEPLOY_RELEASE_REPO%
echo.
echo Download: https://github.com/%DEPLOY_REPO_OWNER%/%DEPLOY_RELEASE_REPO%/releases/download/stable/%DEPLOY_SELF%
endlocal & exit /b 0
