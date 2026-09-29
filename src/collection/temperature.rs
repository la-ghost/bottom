//! Data collection for temperature metrics.
//!
//! For Linux, this is handled by custom code.
//! For everything else, this is handled by sysinfo.

cfg_select! {
    target_os = "linux" => {
        pub mod linux;
        pub use self::linux::*;
    }
    target_os = "windows" => {
        pub mod sysinfo;
        pub mod windows;
        pub use self::windows::*;
    }
    _ => {
        pub mod sysinfo;
        pub use self::sysinfo::*;
    }
}

#[derive(Default, Debug, Clone)]
pub struct TempSensorData {
    /// The name of the sensor.
    pub name: String,

    /// The temperature in Celsius.
    pub temperature: Option<f32>,
}
