@echo off
setlocal EnableExtensions DisableDelayedExpansion
title WTO + YSM Auto Deploy

if /I "%~1"=="--help" goto help
if /I "%~1"=="/?" goto help

set "MOD_ROOT=%~dp0"
if "%MOD_ROOT:~-1%"=="\" set "MOD_ROOT=%MOD_ROOT:~0,-1%"
set "YMB_DIR=%MOD_ROOT%\YMB"
set "YSM_DIR=%YMB_DIR%\mods\YSM"
set "WTO_DIR=%YMB_DIR%\mods\WTO"
set "YMB_REPOSITORY=https://github.com/Yokaiste/YMB"
set "YSM_REPOSITORY=https://github.com/Yokaiste/YSM.git"
set "WTO_REPOSITORY=https://github.com/Yokaiste/WTO.git"

echo.
echo ============================================================
echo   WTO + YSM Auto Deploy
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

call :deploy_repository "YSM" "%YSM_DIR%" "%YSM_REPOSITORY%" "YSM" "4/5"
if errorlevel 1 goto failed

call :deploy_repository "WTO" "%WTO_DIR%" "%WTO_REPOSITORY%" "WTO" "5/5"
if errorlevel 1 goto failed

call :build_preview
if errorlevel 1 goto failed

call :cleanup
echo.
echo ============================================================
echo   WTO and YSM are ready to sync
echo ============================================================
echo.
echo YMB built and validated a combined preview. It did not change live WARNO files.
echo Review the preview under:
echo   "%YMB_DIR%\.ymb-build\output"
echo.
echo To apply WTO with YSM, run this from any terminal:
echo   "%YMB_DIR%\YMB.bat" sync --mod ysm --mod wto --yes
echo.
echo To restore the files saved by YMB later:
echo   "%YMB_DIR%\YMB.bat" recover --mod ysm --mod wto --yes
echo.
echo You do not need this deployment file for those commands. You can also
echo double-click "%YMB_DIR%\YMB.bat" and enter the shorter commands:
echo   sync --mod ysm --mod wto --yes
echo   recover --mod ysm --mod wto --yes
echo.
if not defined CI pause
endlocal & exit /b 0

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
echo   4. Copy %~nx0 into the new folder and run it there.
echo.
echo Expected location:
echo   ^<SteamLibrary^>\steamapps\common\WARNO\Mods\YourModName\%~nx0
echo.
echo The same folder must contain both CommonData and GameData.
exit /b 1

:check_tools
where git.exe >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Git is required but was not found on PATH.
  echo Download Git for Windows: https://git-scm.com/download/win
  exit /b 1
)

git lfs version >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Git LFS is required but is not installed.
  echo Download Git LFS: https://git-lfs.com/
  echo Then run: git lfs install
  exit /b 1
)

where powershell.exe >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Windows PowerShell was not found on PATH.
  exit /b 1
)

where tar.exe >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Windows tar.exe was not found on PATH.
  echo Install current Windows updates and retry.
  exit /b 1
)

where robocopy.exe >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Windows robocopy.exe was not found on PATH.
  exit /b 1
)

git lfs install >nul
if errorlevel 1 (
  echo [ERROR] Git LFS could not initialize for this user.
  exit /b 1
)

for /f "delims=" %%V in ('git --version') do echo [OK] %%V
for /f "delims=" %%V in ('git lfs version') do echo [OK] %%V
exit /b 0

:create_temp
set "DEPLOY_TEMP=%TEMP%\wto-auto-deploy-%RANDOM%-%RANDOM%"
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

:download_ymb
echo.
echo [1/5] Finding the latest portable YMB release...
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $headers=@{Accept='application/vnd.github+json'; 'User-Agent'='WTO-Auto-Deploy'}; $release=Invoke-RestMethod -UseBasicParsing -Headers $headers -Uri 'https://api.github.com/repos/Yokaiste/YMB/releases/latest'; $asset=$release.assets ^| Where-Object { $_.name -match '^YMB-v.+-windows-x64\.zip$' -and $_.name -notmatch '-no-bun\.zip$' } ^| Select-Object -First 1; if ($null -eq $asset) { throw 'The full Windows YMB archive is missing from the latest release.' }; [IO.File]::WriteAllText($env:YMB_URL_FILE, $asset.browser_download_url, [Text.UTF8Encoding]::new($false))"
if errorlevel 1 (
  echo [ERROR] Could not find the full YMB archive at %YMB_REPOSITORY%/releases/latest
  exit /b 1
)

set /p "YMB_URL="<"%YMB_URL_FILE%"
if not defined YMB_URL (
  echo [ERROR] The YMB release returned an empty download URL.
  exit /b 1
)

echo [2/5] Downloading portable YMB...
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $ProgressPreference='SilentlyContinue'; Invoke-WebRequest -UseBasicParsing -Uri $env:YMB_URL -OutFile $env:YMB_ARCHIVE"
if errorlevel 1 (
  echo [ERROR] YMB download failed: "%YMB_URL%"
  exit /b 1
)

tar.exe -xf "%YMB_ARCHIVE%" -C "%YMB_EXTRACT%"
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
echo [3/5] Deploying the production YMB package...
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
robocopy.exe "%YMB_EXTRACT%\YMB" "%YMB_DIR%" /E /COPY:DAT /DCOPY:DAT /R:2 /W:1 /NFL /NDL /NJH /NJS /NP >nul
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

:deploy_repository
setlocal
set "REPO_NAME=%~1"
set "REPO_DIR=%~2"
set "REPO_URL=%~3"
set "REPO_SLUG=%~4"
set "REPO_STEP=%~5"

echo [%REPO_STEP%] Cloning or updating %REPO_NAME%...
if not exist "%REPO_DIR%\" goto clone_repository
if not exist "%REPO_DIR%\.git\" (
  echo [ERROR] "%REPO_DIR%" exists but is not a Git checkout.
  echo Move or rename that folder, then run this file again.
  endlocal & exit /b 1
)

set "REPO_REMOTE="
for /f "delims=" %%R in ('git -C "%REPO_DIR%" remote get-url origin 2^>nul') do set "REPO_REMOTE=%%R"
if /I "%REPO_REMOTE%"=="%REPO_URL%" goto check_repository_clean
if /I "%REPO_REMOTE%"=="https://github.com/Yokaiste/%REPO_SLUG%" goto check_repository_clean
if /I "%REPO_REMOTE%"=="https://github.com/Yokaiste/%REPO_SLUG%.git" goto check_repository_clean
if /I "%REPO_REMOTE%"=="git@github.com:Yokaiste/%REPO_SLUG%.git" goto check_repository_clean

echo [ERROR] Existing %REPO_NAME% checkout has an unexpected origin:
echo   %REPO_REMOTE%
echo Expected: %REPO_URL%
endlocal & exit /b 1

:check_repository_clean
git -C "%REPO_DIR%" fetch --prune origin
if errorlevel 1 (
  echo [ERROR] Existing %REPO_NAME% could not fetch its remote state.
  endlocal & exit /b 1
)

git -C "%REPO_DIR%" remote set-head origin --auto >nul 2>nul
set "REPO_BRANCH="
set "REPO_DEFAULT_BRANCH="
for /f "delims=" %%B in ('git -C "%REPO_DIR%" branch --show-current') do set "REPO_BRANCH=%%B"
for /f "delims=" %%B in ('git -C "%REPO_DIR%" symbolic-ref --short refs/remotes/origin/HEAD 2^>nul') do set "REPO_DEFAULT_BRANCH=%%B"
if /I "origin/%REPO_BRANCH%"=="%REPO_DEFAULT_BRANCH%" goto inspect_repository_status

echo [ERROR] Existing %REPO_NAME% is on "%REPO_BRANCH%", not its default branch.
echo Switch to the default branch without local changes, then retry.
endlocal & exit /b 1

:inspect_repository_status
git -C "%REPO_DIR%" status --porcelain --untracked-files=normal >"%DEPLOY_TEMP%\%REPO_SLUG%-status.txt"
for %%F in ("%DEPLOY_TEMP%\%REPO_SLUG%-status.txt") do if %%~zF GTR 0 goto dirty_repository

git -C "%REPO_DIR%" pull --ff-only
if errorlevel 1 (
  echo [ERROR] Existing %REPO_NAME% could not be updated with a fast-forward pull.
  echo Resolve its branch state manually and retry.
  endlocal & exit /b 1
)
goto fetch_repository_lfs

:dirty_repository
echo [ERROR] Existing %REPO_NAME% contains local changes. Nothing was overwritten.
echo Commit, stash, or remove those changes in:
echo   "%REPO_DIR%"
endlocal & exit /b 1

:clone_repository
git clone "%REPO_URL%" "%REPO_DIR%"
if errorlevel 1 (
  echo [ERROR] Could not clone %REPO_NAME% from %REPO_URL%
  endlocal & exit /b 1
)

:fetch_repository_lfs
git -C "%REPO_DIR%" lfs pull
if errorlevel 1 (
  echo [ERROR] Git LFS could not download all %REPO_NAME% assets.
  endlocal & exit /b 1
)
endlocal & exit /b 0

:build_preview
echo.
echo Validating the installed tools and combined YSM + WTO configuration...
call "%YMB_DIR%\YMB.bat" doctor
if errorlevel 1 exit /b 1

call "%YMB_DIR%\YMB.bat" validate --mod ysm --mod wto
if errorlevel 1 exit /b 1

echo.
echo Building a safe combined preview. Live WARNO files will not be changed...
call "%YMB_DIR%\YMB.bat" build --mod ysm --mod wto
if errorlevel 1 exit /b 1
exit /b 0

:cleanup
if defined DEPLOY_TEMP if exist "%DEPLOY_TEMP%\" rmdir /s /q "%DEPLOY_TEMP%"
exit /b 0

:failed
call :cleanup
echo.
echo Deployment stopped safely. No sync or recovery command was run.
echo Fix the error above, then run %~nx0 again.
echo.
if not defined CI pause
endlocal & exit /b 1

:help
echo WTO + YSM Auto Deploy
echo.
echo Place this file inside a generated WARNO mod folder, beside its
echo CommonData and GameData folders, then double-click it.
echo.
echo It checks Git and Git LFS, downloads the latest portable YMB release,
echo clones or safely updates YSM and WTO, validates both, and builds a
echo combined preview. It never syncs or recovers live WARNO files automatically.
echo.
echo Download: https://github.com/Yokaiste/WTO/releases/download/auto-deploy/Deploy-WTO.bat
endlocal & exit /b 0
