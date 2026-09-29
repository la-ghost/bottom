$ErrorActionPreference = 'Stop'
$btm = Join-Path $PSScriptRoot 'btm.exe'
$terminal = Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps\wt.exe'

if (-not (Test-Path -LiteralPath $btm)) {
    throw "btm.exe was not found beside this launcher."
}

if (-not (Test-Path -LiteralPath $terminal)) {
    $terminal = 'wt.exe'
}

$terminalArguments = '-w new nt --title "Bottom System Monitor - Temperatures" -d "' +
    $PSScriptRoot + '" cmd.exe /k ""' + $btm + '""'

# LibreHardwareMonitor needs an elevated process to read CPU and motherboard
# sensors. Elevating Windows Terminal also lets the sensor host inherit the
# required access without a second prompt.
Start-Process -FilePath $terminal -Verb RunAs -ArgumentList $terminalArguments
