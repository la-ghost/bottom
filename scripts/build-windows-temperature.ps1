[CmdletBinding()]
param(
    [switch]$FrameworkDependent
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$sensorOutput = Join-Path $repoRoot 'target\windows-sensors-publish'
$distribution = Join-Path $repoRoot 'dist\bottom-windows-temperature'

Push-Location $repoRoot
try {
    cargo build --release --all-features

    $selfContained = if ($FrameworkDependent) { 'false' } else { 'true' }
    dotnet publish '.\windows-sensors\Bottom.SensorHost.csproj' `
        -c Release `
        -r win-x64 `
        --self-contained $selfContained `
        -o $sensorOutput

    New-Item -ItemType Directory -Path $distribution -Force | Out-Null
    Copy-Item -LiteralPath '.\target\release\btm.exe' -Destination $distribution -Force
    Copy-Item -Path (Join-Path $sensorOutput '*') -Destination $distribution -Recurse -Force
    Copy-Item -LiteralPath '.\WINDOWS_TEMPERATURES.md' -Destination $distribution -Force
    Copy-Item -LiteralPath '.\windows-sensors\Start Bottom with Temperatures.ps1' -Destination $distribution -Force

    Write-Host "Built: $distribution"
} finally {
    Pop-Location
}
