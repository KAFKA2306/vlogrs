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
    param([string]$Message, [string]$Level = "INFO", [bool]$Quiet = $false)
    $Timestamp = Get-Date -Format "yyyy/MM/dd HH:mm:ss.ff"
    $LogLine = "[$Timestamp] [$Level] $Message"
    if (!$Quiet -or $Level -eq "FATAL" -or $Level -eq "ERROR") {
        Write-Host "[$Level] $Message"
    }
    $LogLine | Out-File -FilePath $BootstrapLog -Append -Encoding utf8
}

Write-Log "--- VLog Bootstrap Started ---" $null $true
Write-Log "Base: $BaseDir" "DEBUG" $true

$MaxConnectRetries = 5
$ConnectDelay = 1
for ($i = 0; $i -lt $MaxConnectRetries; $i++) {
    if (Test-Path $BaseDir) { break }
    Write-Log "Connecting to shared storage..." "WARN"
    Start-Sleep -Seconds $ConnectDelay
    if ($i -eq $MaxConnectRetries - 1) {
        Write-Log "Storage unreachable." "FATAL"
        exit 1
    }
}

$CargoPath = (Get-Command cargo -ErrorAction SilentlyContinue).Source
if (!$CargoPath) {
    $UserCargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path $UserCargo) { $CargoPath = $UserCargo }
}
if (!$CargoPath) {
    Write-Log "Cargo missing." "FATAL"
    exit 1
}
else {
    $CargoDir = Split-Path $CargoPath
    if ($env:PATH -notmatch [regex]::Escape($CargoDir)) { $env:PATH = "$CargoDir;$env:PATH" }
}

# Check application status
$VrcRunning = Get-Process -Name "VRChat" -ErrorAction SilentlyContinue
$DiscordRunning = Get-Process -Name "Discord" -ErrorAction SilentlyContinue
$VrcStatus = if ($VrcRunning) { "Online" } else { "Offline" }
$DiscordStatus = if ($DiscordRunning) { "Online" } else { "Offline" }
Write-Host "[INFO] VRChat: $VrcStatus / Discord: $DiscordStatus"

Get-Process -Name "vlog-rs" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Write-Host "[INFO] Building VLog..."
Push-Location $BaseDir
$OldEAP = $ErrorActionPreference
$ErrorActionPreference = "Continue"

& $CargoPath build --release --target x86_64-pc-windows-msvc 2>&1 | Out-File -FilePath $BuildLog -Encoding utf8
$BuildExitCode = $LASTEXITCODE
$ErrorActionPreference = $OldEAP

if ($BuildExitCode -ne 0) {
    Write-Log "Build failed (code $BuildExitCode). See build.log" "ERROR"
    Pop-Location
    exit 1
}

if (!(Test-Path $AgentExe)) {
    Write-Log "Binary missing: $AgentExe" "FATAL"
    Pop-Location
    exit 1
}

Write-Host "[INFO] Launching Monitor..."
$IsStandby = !($VrcRunning -or $DiscordRunning)
if (-not $IsStandby) {
    Write-Host "[INFO] Status: Preparing to record..." -ForegroundColor Yellow
} else {
    Write-Host "[INFO] Status: Standby (Not Recording)" -ForegroundColor DarkGray
}

Push-Location $BaseDir
& $AgentExe "monitor" 2>&1 | ForEach-Object {
    $Line = $_.ToString()
    # Log all to file
    $Line | Out-File -FilePath $MonitorLog -Append -Encoding utf8
    
    # Passthrough specific events to console
    if ($Line -match "Recording started") {
        Write-Host "[INFO] Recording started." -ForegroundColor Cyan
        $IsStandby = $false
    }
    elseif ($Line -match "Session recording saved to") {
        $FileName = if ($Line -match "([^\\/]+\.wav)") { $Matches[1] } else { "session" }
        Write-Host "[INFO] Recording saved: $FileName" -ForegroundColor Green
        Write-Host "[INFO] Status: Standby (Not Recording)" -ForegroundColor DarkGray
        $IsStandby = $true
    }
    elseif ($Line -match "Stop trigger pending" -and !$IsStandby) {
        Write-Host "[INFO] Status: Waiting for stop grace period..." -ForegroundColor Yellow
    }
}
$ExitCode = $LASTEXITCODE
Pop-Location

exit $ExitCode
