# ---------------------------------------------------------
# VLog Bootstrap Script (Master Protocol v1.0)
# ---------------------------------------------------------
# This script is the single point of execution logic for the Windows VLog agent.
# It should be invoked via windows/run.bat.

$ErrorActionPreference = "Stop"

# 1. Resolve Paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$BaseDir = Resolve-Path (Join-Path $ScriptDir "../../../") | Select-Object -ExpandProperty ProviderPath
$LogDir = Join-Path $BaseDir "logs"
$BootstrapLog = Join-Path $LogDir "windows-rust-bootstrap.log"
$MonitorLog = Join-Path $LogDir "windows-rust-monitor.log"
$BuildLog = Join-Path $LogDir "build.log"
$AgentExe = Join-Path $BaseDir "target\release\vlog-rs.exe"

# Ensure Log Directory
if (!(Test-Path $LogDir)) { New-Item -ItemType Directory -Path $LogDir | Out-Null }

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $Timestamp = Get-Date -Format "yyyy/MM/dd HH:mm:ss.ff"
    $LogLine = "[$Timestamp] [$Level] $Message"
    Write-Host $LogLine
    $LogLine | Out-File -FilePath $BootstrapLog -Append -Encoding utf8
}

Write-Log "--- VLog Bootstrap Started ---"
Write-Log "Base Directory: $BaseDir"

# 2. Check Environment
$CargoPath = (Get-Command cargo -ErrorAction SilentlyContinue).Source
if (!$CargoPath) {
    $UserCargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path $UserCargo) {
        $CargoPath = $UserCargo
    } else {
        Write-Log "Cargo not found. Please install Rust toolchain." "FATAL"
        exit 1
    }
}
Write-Log "Using Cargo: $CargoPath"

# 3. Build / Check Agent
Write-Log "Ensuring clean state..."
Get-Process -Name "vlog-rs" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Write-Log "Initiating build..."
try {
    Push-Location $BaseDir
    & $CargoPath build --release 2>&1 | Out-File -FilePath $BuildLog -Encoding utf8
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Build failed. Check $BuildLog for details." "FATAL"
        exit 1
    }
    Write-Log "Build successful."
} finally {
    Pop-Location
}

# 4. Pre-flight Check
Write-Log "Checking target applications..."
$Discord = Get-Process -Name "Discord" -ErrorAction SilentlyContinue
$VRChat = Get-Process -Name "VRChat" -ErrorAction SilentlyContinue

if ($Discord) { Write-Log "Discord: RUNNING" } else { Write-Log "Discord: NOT DETECTED" }
if ($VRChat) { Write-Log "VRChat: RUNNING" } else { Write-Log "VRChat: NOT DETECTED" }

# 5. Run Agent Loop
Write-Log "Launching Master Monitor..."
if (!(Test-Path $AgentExe)) {
    Write-Log "Binary not found: $AgentExe" "FATAL"
    exit 1
}

$RestartDelay = 5
while ($true) {
    Write-Log "Starting Monitor: $AgentExe"
    try {
        Push-Location $BaseDir
        # Run process and capture output to monitor log
        Start-Process -FilePath $AgentExe -ArgumentList "monitor" -NoNewWindow -Wait -PassThru | Out-Null
        $ExitCode = $LASTEXITCODE
        Write-Log "Monitor exited with code: $ExitCode" "WARN"
    } catch {
        Write-Log "Failed to launch monitor: $_" "ERROR"
    } finally {
        Pop-Location
    }

    Write-Log "Restarting in $RestartDelay seconds..."
    Start-Sleep -Seconds $RestartDelay
}
