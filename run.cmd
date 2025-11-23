@echo off
set "PS7=%ProgramFiles%\PowerShell\7\pwsh.exe"
if exist "%PS7%" (
  "%PS7%" -NoProfile -ExecutionPolicy Bypass -File "%~dp0run.ps1"
) else (
  powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0run.ps1"
)
