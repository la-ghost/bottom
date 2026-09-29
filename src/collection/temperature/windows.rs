//! Windows temperature collection through the bundled LibreHardwareMonitor host.
//!
//! Windows often does not expose useful hardware temperatures through the APIs
//! used by `sysinfo`. This module talks to a small, long-lived .NET companion
//! process and falls back to `sysinfo` if that process is unavailable.

use std::{
    env,
    io::Write,
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use super::TempSensorData;
use crate::app::filter::Filter;

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Deserialize)]
struct SensorHostReading {
    name: String,
    temperature: f32,
}

#[derive(Debug, Default)]
pub struct WindowsTemperatureSource {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    responses: Option<Receiver<String>>,
}

impl WindowsTemperatureSource {
    fn helper_path() -> Result<PathBuf> {
        if let Some(path) = env::var_os("BTM_SENSOR_HOST") {
            return Ok(PathBuf::from(path));
        }

        let mut path = env::current_exe().context("cannot locate btm executable")?;
        path.set_file_name("btm-sensor-host.exe");
        Ok(path)
    }

    fn start(&mut self) -> Result<()> {
        use std::{
            io::{BufRead, BufReader},
            os::windows::process::CommandExt,
            thread,
        };

        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        let path = Self::helper_path()?;
        if !path.is_file() {
            return Err(anyhow!(
                "Windows temperature sensor host is missing at {}",
                path.display()
            ));
        }

        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .context("cannot start Windows temperature sensor host")?;

        let stdin = child
            .stdin
            .take()
            .context("sensor host stdin unavailable")?;
        let stdout = child
            .stdout
            .take()
            .context("sensor host stdout unavailable")?;
        let (sender, receiver) = mpsc::channel();

        thread::Builder::new()
            .name("btm-temperature-host".into())
            .spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            if sender.send(line).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .context("cannot start sensor host reader")?;

        self.child = Some(child);
        self.stdin = Some(stdin);
        self.responses = Some(receiver);
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(stdin) = &mut self.stdin {
            let _ = stdin.write_all(b"quit\n");
            let _ = stdin.flush();
        }
        self.stdin = None;
        self.responses = None;
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn read(&mut self) -> Result<Vec<SensorHostReading>> {
        if self.child.is_none() {
            self.start()?;
        }

        let result = (|| {
            let stdin = self
                .stdin
                .as_mut()
                .context("sensor host stdin unavailable")?;
            stdin.write_all(b"refresh\n")?;
            stdin.flush()?;

            let line = self
                .responses
                .as_ref()
                .context("sensor host response channel unavailable")?
                .recv_timeout(RESPONSE_TIMEOUT)
                .context("sensor host did not respond")?;

            serde_json::from_str(&line).context("sensor host returned invalid JSON")
        })();

        if result.is_err() {
            self.stop();
        }
        result
    }
}

impl Drop for WindowsTemperatureSource {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn get_temperature_data(
    source: &mut WindowsTemperatureSource, components: &sysinfo::Components,
    filter: &Option<Filter>, graph_filter: &Option<Filter>,
) -> Result<Option<Vec<TempSensorData>>> {
    if let Ok(readings) = source.read() {
        let temperatures = readings
            .into_iter()
            .filter(|reading| {
                reading.temperature.is_finite()
                    && reading.temperature > 0.0
                    && reading.temperature <= 100.0
                    && !reading.name.contains("Distance to TjMax")
                    && (Filter::optional_should_keep(filter, &reading.name)
                        || Filter::optional_should_keep(graph_filter, &reading.name))
            })
            .map(|reading| TempSensorData {
                name: reading.name,
                temperature: Some(reading.temperature),
            })
            .collect::<Vec<_>>();

        if !temperatures.is_empty() {
            return Ok(Some(temperatures));
        }
    }

    super::sysinfo::get_temperature_data(components, filter, graph_filter)
}
