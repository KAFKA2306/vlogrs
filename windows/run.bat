@echo off
setlocal EnableExtensions EnableDelayedExpansion

:: ---------------------------------------------------------
:: VLog Windows Bootstrap Wrapper (Master Protocol v8.0)
:: ---------------------------------------------------------

:: Handle UNC paths - pushd will assign a temporary drive letter if needed
pushd "%~dp0" >nul 2>&1
if errorlevel 1 (
  echo [FATAL] Cannot access script directory from UNC path.
  pause
  exit /b 1
)

:: Move to Repo Root
pushd ".." >nul 2>&1
set "REPO_ROOT=%CD%"
popd >nul 2>&1

:: Resolve bootstrap script path
set "BOOTSTRAP_PS1=%REPO_ROOT%\src\windows\rust\bootstrap.ps1"

if not exist "%BOOTSTRAP_PS1%" (
    echo [FATAL] Bootstrap script not found: %BOOTSTRAP_PS1%
    pause
    exit /b 1
)

:: Invoke PowerShell with the bootstrap logic
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%BOOTSTRAP_PS1%"

:: Capture exit code
set "PS_EXIT_CODE=%ERRORLEVEL%"

popd
exit /b %PS_EXIT_CODE%
