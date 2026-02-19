@echo off
setlocal EnableExtensions EnableDelayedExpansion

:: ---------------------------------------------------------
:: VLog Windows Bootstrap Script (Master Protocol v7.2)
:: ---------------------------------------------------------

:: Handle UNC paths
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

:: Configuration
set "LOG_DIR=%REPO_ROOT%\logs"
set "BOOTSTRAP_LOG=%LOG_DIR%\windows-rust-bootstrap.log"
set "MONITOR_LOG=%LOG_DIR%\windows-rust-monitor.log"
:: Use main vlog-rs binary as per Skill Manual
set "AGENT_EXE=%REPO_ROOT%\target\release\vlog-rs.exe"

:: Ensure Log Directory
if not exist "%LOG_DIR%" mkdir "%LOG_DIR%"

:: Logging wrapper (Truncate log at start of session to avoid hallucination)
if not exist "%BOOTSTRAP_LOG%" ( echo. > "%BOOTSTRAP_LOG%" ) else ( type nul > "%BOOTSTRAP_LOG%" )

call :log "--- VLog Bootstrap Started ---"
call :log "Repo Root: %REPO_ROOT%"

:: ---------------------------------------------------------
:: Build / Check Rust Agent
:: ---------------------------------------------------------
:check_build
call :log "Ensuring clean state..."
taskkill /F /IM vlog-rs.exe >nul 2>&1
call :log "Initiating build..."
call :find_cargo
if "%CARGO_PATH%"=="" (
    call :log "[FATAL] Cargo not found. Please install Rust toolchain on Windows."
    timeout /t 30
    exit /b 1
)
call :log "Using Cargo: %CARGO_PATH%"

:: Ensure UTF-8 for Japanese logs
chcp 65001 >nul

pushd "%REPO_ROOT%"
call :log "Building vlog-rs (Master Engine)..."
:: Build root project - explicitly use the found cargo path
"%CARGO_PATH%" build --release
if errorlevel 1 (
    call :log "[FATAL] Build failed."
    popd
    timeout /t 30
    exit /b 1
)
popd
call :log "Build successful."

:: ---------------------------------------------------------
:: Pre-flight Check (Target Status)
:: ---------------------------------------------------------
call :log "Checking target applications..."
tasklist /FI "IMAGENAME eq Discord.exe" 2>NUL | find /I /N "Discord.exe">NUL
if "%ERRORLEVEL%"=="0" ( call :log "[PRE-FLIGHT] Discord: RUNNING" ) else ( call :log "[PRE-FLIGHT] Discord: NOT DETECTED" )

tasklist /FI "IMAGENAME eq VRChat.exe" 2>NUL | find /I /N "VRChat.exe">NUL
if "%ERRORLEVEL%"=="0" ( call :log "[PRE-FLIGHT] VRChat: RUNNING" ) else ( call :log "[PRE-FLIGHT] VRChat: NOT DETECTED" )

:: ---------------------------------------------------------
:: Run Agent
:: ---------------------------------------------------------
:run_agent
if not exist "%AGENT_EXE%" (
    call :log "[FATAL] Binary not found: %AGENT_EXE%"
    goto :check_build
)

call :log "Launching Master Monitor: %AGENT_EXE%"

pushd "%REPO_ROOT%"
:: Start the Rust agent with 'monitor' command
:: Use direct execution to avoid PowerShell pipeline masking of the main exit code
"%AGENT_EXE%" monitor
set "EXIT_CODE=%ERRORLEVEL%"
popd

if "%EXIT_CODE%"=="0" (
    call :log "Monitor exited normally."
    exit /b 0
)

call :log "[WARN] Monitor crashed with code %EXIT_CODE%. Restarting in 5 seconds..."
timeout /t 5 /nobreak >nul
goto :run_agent

:: ---------------------------------------------------------
:: Utilities
:: ---------------------------------------------------------
:find_cargo
set "CARGO_PATH="
:: Take the first match from 'where'
for /f "delims=" %%I in ('where cargo 2^>nul') do (
    if "!CARGO_PATH!"=="" set "CARGO_PATH=%%I"
)
if "%CARGO_PATH%"=="" (
    if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "CARGO_PATH=%USERPROFILE%\.cargo\bin\cargo.exe"
)
exit /b

:log
echo [%DATE% %TIME%] %~1
echo [%DATE% %TIME%] %~1 >> "%BOOTSTRAP_LOG%"
exit /b
