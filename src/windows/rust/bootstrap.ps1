$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$BaseDir = Resolve-Path (Join-Path $ScriptDir "../../../") | Select-Object -ExpandProperty ProviderPath
$LogDir = Join-Path $BaseDir "logs"
$BootstrapLog = Join-Path $LogDir "windows-rust-bootstrap.log"
$MonitorLog = Join-Path $LogDir "windows-rust-monitor.log"
$BuildLog = Join-Path $LogDir "build.log"
$AgentExe = Join-Path $BaseDir "target\x86_64-pc-windows-msvc\release\vlog-rs.exe"

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

Write-Log "Checking Shared Folder connection..."
$MaxConnectRetries = 10
$ConnectDelay = 2
for ($i = 0; $i -lt $MaxConnectRetries; $i++) {
    if (Test-Path $BaseDir) {
        Write-Log "Connection verified: $BaseDir"
        break
    }
    Write-Log "Base directory not found. Retrying in $ConnectDelay seconds... ($($i+1)/$MaxConnectRetries)" "WARN"
    Start-Sleep -Seconds $ConnectDelay
    $ConnectDelay *= 2
    if ($i -eq $MaxConnectRetries - 1) {
        Write-Log "Could not verify connection to $BaseDir after $MaxConnectRetries attempts." "FATAL"
        exit 1
    }
}

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

Write-Log "Ensuring clean state..."
Get-Process -Name "vlog-rs" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Write-Log "Initiating build..."
Push-Location $BaseDir
$OldEAP = $ErrorActionPreference
$ErrorActionPreference = "Continue"

# Run build
& $CargoPath build --release --target x86_64-pc-windows-msvc 2>&1 | Out-File -FilePath $BuildLog -Encoding utf8
$BuildExitCode = $LASTEXITCODE
$ErrorActionPreference = $OldEAP

if ($BuildExitCode -ne 0) {
    Write-Log "Build failed. Check $BuildLog for details." "ERROR"
    Pop-Location
    exit 1
}

Write-Log "Build successful. Launching Master Monitor..."
if (!(Test-Path $AgentExe)) {
    Write-Log "Binary not found: $AgentExe" "FATAL"
    Pop-Location
    exit 1
}

Write-Log "Starting Monitor: $AgentExe"
Push-Location $BaseDir
& $AgentExe "monitor" 2>&1 | Out-File -FilePath $MonitorLog -Encoding utf8
$ExitCode = $LASTEXITCODE
Write-Log "Monitor exited with code: $ExitCode" "WARN"
Pop-Location

exit $ExitCode

