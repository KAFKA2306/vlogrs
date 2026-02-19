@echo off
setlocal EnableExtensions EnableDelayedExpansion

set "SCRIPT_DIR=%~dp0"
pushd "%SCRIPT_DIR%" >nul 2>&1
if errorlevel 1 (
  echo Failed to access script directory: %SCRIPT_DIR%
  exit /b 1
)

pushd ".." >nul 2>&1
set "REPO_ROOT=%CD%"
popd >nul 2>&1

set "LOG_DIR=%REPO_ROOT%\logs"
set "BOOTSTRAP_LOG=%LOG_DIR%\windows-rust-bootstrap.log"
set "MONITOR_LOG=%LOG_DIR%\windows-rust-monitor.log"
set "MONITOR_EXE=%REPO_ROOT%\target\release\vlog-rs.exe"
set "CARGO_PATH="
set "FORCE_BUILD=0"
if /I "%~1"=="rebuild" set "FORCE_BUILD=1"

if not exist "%LOG_DIR%" mkdir "%LOG_DIR%" >nul 2>&1

call :log "VLog Windows Rust Monitor Bootstrapper Starting..."
call :log "Repo: %REPO_ROOT%"
call :log "Bootstrap log: %BOOTSTRAP_LOG%"
call :log "Monitor log: %MONITOR_LOG%"
call :log "Monitor exe: %MONITOR_EXE%"

:main_loop
set "NEED_BUILD=0"
if "%FORCE_BUILD%"=="1" set "NEED_BUILD=1"
if "%FORCE_BUILD%"=="1" call :log "Force rebuild requested."
if not exist "%MONITOR_EXE%" set "NEED_BUILD=1"

if "%NEED_BUILD%"=="1" (
  call :resolve_cargo
  if not defined CARGO_PATH (
    call :log "cargo.exe not found. Retrying in 10 seconds."
    timeout /t 10 /nobreak >nul
    goto main_loop
  )

  call :log "Monitor binary not found. Building release binary..."
  pushd "%REPO_ROOT%" >nul 2>&1
  "%CARGO_PATH%" build --release >> "%MONITOR_LOG%" 2>&1
  set "BUILD_EXIT=%ERRORLEVEL%"
  popd >nul 2>&1
  if not "%BUILD_EXIT%"=="0" (
    call :log "Release build failed (exit=%BUILD_EXIT%). Retrying in 10 seconds."
    call :tail_monitor
    timeout /t 10 /nobreak >nul
    goto main_loop
  )

  if not exist "%MONITOR_EXE%" (
    call :log "Release build finished but monitor binary is missing. Retrying in 10 seconds."
    timeout /t 10 /nobreak >nul
    goto main_loop
  )
  call :log "Release binary ready: %MONITOR_EXE%"
  set "FORCE_BUILD=0"
)

call :log "Launching: \"%MONITOR_EXE%\" monitor"
call :log "Launch context: cwd=%REPO_ROOT% cmd=target\release\vlog-rs.exe monitor"
pushd "%REPO_ROOT%" >nul 2>&1
"%MONITOR_EXE%" monitor >> "%MONITOR_LOG%" 2>&1
set "MON_EXIT=%ERRORLEVEL%"
popd >nul 2>&1

if "%MON_EXIT%"=="0" (
  call :log "Monitor exited normally."
  goto finish
)
if "%MON_EXIT%"=="130" (
  call :log "Monitor stopped by user signal (exit code: %MON_EXIT%)."
  goto finish
)
if "%MON_EXIT%"=="3221225786" (
  call :log "Monitor stopped by user signal (exit code: %MON_EXIT%)."
  goto finish
)

call :log "Monitor crashed: monitor process exited with code %MON_EXIT%"
call :tail_monitor
call :log "Restarting in 5 seconds..."
timeout /t 5 /nobreak >nul
goto main_loop

:resolve_cargo
if defined CARGO_PATH goto :eof
if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
  set "CARGO_PATH=%USERPROFILE%\.cargo\bin\cargo.exe"
  call :log "Resolved cargo path: %CARGO_PATH%"
  goto :eof
)
for /f "delims=" %%I in ('where cargo 2^>nul') do (
  set "CARGO_PATH=%%I"
  call :log "Resolved cargo path: !CARGO_PATH!"
  goto :eof
)
goto :eof

:tail_monitor
if not exist "%MONITOR_LOG%" goto :eof
call :log "Monitor stderr/stdout tail (last 20 lines):"
powershell -NoLogo -NoProfile -Command ^
  "Get-Content -Path '%MONITOR_LOG%' -Tail 20 | ForEach-Object { '[monitor-tail] ' + $_ }" >> "%BOOTSTRAP_LOG%" 2>&1
goto :eof

:log
set "MSG=%~1"
for /f "usebackq delims=" %%T in (`powershell -NoLogo -NoProfile -Command "(Get-Date).ToString('yyyy-MM-dd HH:mm:ss')"`) do set "TS=%%T"
set "LINE=[!TS!] !MSG!"
echo !LINE!
>> "%BOOTSTRAP_LOG%" echo !LINE!
goto :eof

:finish
popd >nul 2>&1
endlocal
