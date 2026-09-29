# Windows temperature sensor host

`btm-sensor-host` is a small Windows-only companion process for this fork of
bottom. It uses LibreHardwareMonitor to read CPU, GPU, motherboard, memory, and
storage temperature sensors that Windows does not expose through `sysinfo`.

The host accepts one line on standard input per refresh and returns one JSON
array on standard output. Keeping it alive avoids reinitializing hardware and
drivers every second. The main `btm` process starts it only when a temperature
widget is active and falls back to `sysinfo` if the host is absent or fails.

Build it with:

```powershell
dotnet publish .\windows-sensors\Bottom.SensorHost.csproj -c Release -r win-x64 --self-contained false
```

Place the published files beside `btm.exe`. Hardware access generally requires
running the launcher as administrator. The supplied Windows packaging script
creates the expected layout.
