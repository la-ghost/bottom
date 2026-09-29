# Windows hardware temperatures

This fork adds reliable Windows temperature collection to bottom while keeping
the normal bottom interface and its existing cross-platform collectors.

## Why this exists

Windows frequently exposes no useful CPU, GPU, motherboard, or drive
temperatures through the API used by `sysinfo`. On those machines, upstream
bottom correctly draws the temperature widget but has no data to display.

This fork includes a small Windows-only sensor host built on
[LibreHardwareMonitor](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor).
`btm` starts the host only when a temperature widget is active, keeps it alive,
and exchanges newline-delimited JSON over redirected standard input/output.
If the host is missing, fails, or returns no usable sensors, bottom falls back
to its original `sysinfo` collector.

## Safety and filtering

The sensor host is read-only. Fan/controller support is deliberately disabled.
It collects temperature sensors only and filters:

- missing, non-finite, zero, and negative values;
- values over 100 degrees Celsius, which are typically invalid firmware inputs;
- `Distance to TjMax`, which is thermal headroom rather than a temperature.

LibreHardwareMonitor usually needs administrator rights for low-level hardware
access. Launch the packaged build through an administrator shortcut or run the
terminal as administrator. A UAC prompt is therefore expected.

The packaged `Start Bottom with Temperatures.ps1` launcher elevates Windows
Terminal and starts `btm.exe`; it does not change UAC or security settings.

## Build

Requirements:

- Rust 1.95 or newer
- .NET 10 SDK
- PowerShell 7 or Windows PowerShell

Run:

```powershell
.\scripts\build-windows-temperature.ps1
```

The complete application is written to `dist\bottom-windows-temperature`.
Keep every file in that directory together; `btm.exe` locates
`btm-sensor-host.exe` beside itself. To use a different host location, set the
`BTM_SENSOR_HOST` environment variable.

## Updating from upstream

```powershell
git fetch upstream
git switch codex/windows-libre-temperature
git rebase upstream/main
.\scripts\build-windows-temperature.ps1
```

Resolve changes around `src/collection/temperature.rs`,
`src/collection/temperature/windows.rs`, and `src/collection.rs` carefully.
The GitHub Actions workflow also builds a downloadable Windows artifact.
