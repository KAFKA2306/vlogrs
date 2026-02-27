@echo off
setlocal EnableExtensions EnableDelayedExpansion

pushd "%~dp0" >nul 2>&1
cls
if errorlevel 1 (
  echo [FATAL] UNC path access failed.
  pause
  exit /b 1
)

pushd ".." >nul 2>&1
set "REPO_ROOT=%CD%"
popd >nul 2>&1

set "BOOTSTRAP_PS1=%REPO_ROOT%\src\windows\rust\bootstrap.ps1"

if not exist "%BOOTSTRAP_PS1%" (
    echo [FATAL] Bootstrap missing: %BOOTSTRAP_PS1%
    pause
    exit /b 1
)

echo [INFO] VLog Master Protocol v8.0 Starting...

:loop
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%BOOTSTRAP_PS1%"
set "PS_EXIT_CODE=%ERRORLEVEL%"

if %PS_EXIT_CODE% neq 0 (
    echo [WARN] Monitor exited (%PS_EXIT_CODE%). Restarting in 5s...
)

timeout /t 5 /nobreak >nul
goto loop

popd
exit /b %PS_EXIT_CODE%
