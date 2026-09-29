using System.Text.Json;
using LibreHardwareMonitor.Hardware;

namespace Bottom.SensorHost;

internal sealed record TemperatureReading(string Name, float Temperature);

internal sealed class UpdateVisitor : IVisitor
{
    public void VisitComputer(IComputer computer) => computer.Traverse(this);
    public void VisitHardware(IHardware hardware)
    {
        hardware.Update();
        foreach (var subHardware in hardware.SubHardware)
        {
            subHardware.Accept(this);
        }
    }
    public void VisitSensor(ISensor sensor) { }
    public void VisitParameter(IParameter parameter) { }
}

internal static class Program
{
    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web);

    private static List<TemperatureReading> ReadTemperatures(Computer computer)
    {
        computer.Accept(new UpdateVisitor());
        var readings = new List<TemperatureReading>();

        foreach (var hardware in computer.Hardware)
        {
            AddHardwareTemperatures(hardware, readings);
            foreach (var subHardware in hardware.SubHardware)
            {
                AddHardwareTemperatures(subHardware, readings);
            }
        }

        return readings
            .GroupBy(reading => reading.Name, StringComparer.OrdinalIgnoreCase)
            .Select(group => group.First())
            .OrderBy(reading => reading.Name, StringComparer.OrdinalIgnoreCase)
            .ToList();
    }

    private static void AddHardwareTemperatures(
        IHardware hardware,
        ICollection<TemperatureReading> readings)
    {
        var hardwareName = new string(hardware.Name.Where(character => !char.IsControl(character)).ToArray()).Trim();
        if (string.IsNullOrWhiteSpace(hardwareName))
        {
            return;
        }

        foreach (var sensor in hardware.Sensors)
        {
            if (sensor.SensorType == SensorType.Temperature
                && sensor.Value is float value
                && float.IsFinite(value)
                && value > 0
                && value <= 100
                && !sensor.Name.Contains("Distance to TjMax", StringComparison.OrdinalIgnoreCase))
            {
                readings.Add(new TemperatureReading($"{hardwareName}: {sensor.Name}", value));
            }
        }
    }

    private static void WriteReading(Computer computer)
    {
        Console.WriteLine(JsonSerializer.Serialize(ReadTemperatures(computer), JsonOptions));
        Console.Out.Flush();
    }

    public static int Main(string[] args)
    {
        try
        {
            var computer = new Computer
            {
                IsCpuEnabled = true,
                IsGpuEnabled = true,
                IsMotherboardEnabled = true,
                IsStorageEnabled = true,
                IsMemoryEnabled = true,
                IsControllerEnabled = false,
                IsNetworkEnabled = false,
                IsPsuEnabled = false,
            };

            try
            {
                computer.Open();

                if (args.Contains("--once", StringComparer.OrdinalIgnoreCase))
                {
                    WriteReading(computer);
                    return 0;
                }

                string? request;
                while ((request = Console.ReadLine()) is not null)
                {
                    if (request.Equals("quit", StringComparison.OrdinalIgnoreCase))
                    {
                        break;
                    }

                    WriteReading(computer);
                }

                return 0;
            }
            finally
            {
                computer.Close();
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine(ex);
            return 1;
        }
    }
}
