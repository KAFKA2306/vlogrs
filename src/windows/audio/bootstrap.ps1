# Milestone 26: bootstrap.ps1 for Windows Audio Integration
$env:VLOG_INBOX_DIR = "\\wsl.localhost\Ubuntu\home\kafka\vlog\data\recordings"
$env:UV_PYTHON_PREFERENCE = "only-managed"

Write-Host "VLog Windows Audio Bootstrapper Starting..." -ForegroundColor Cyan

while ($true) {
    Write-Host "Launching Audio Recorder via uv..." -ForegroundColor Green
    try {
        # Milestone 29: Connectivity check
        if (!(Test-Path $env:VLOG_INBOX_DIR)) {
            Write-Host "Waiting for shared folder: $($env:VLOG_INBOX_DIR)..." -ForegroundColor Yellow
            Start-Sleep -Seconds 5
            continue
        }

        uv run src/windows/audio/scripts/recorder.py
    } catch {
        Write-Host "Process crashed: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    Write-Host "Restarting in 5 seconds (Crash-Only Design)..." -ForegroundColor Gray
    Start-Sleep -Seconds 5
}
