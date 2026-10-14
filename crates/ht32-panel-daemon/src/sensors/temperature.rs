//! CPU temperature sensor.

use super::Sensor;
use std::fs;
use std::path::PathBuf;
use tracing::debug;

/// CPU temperature sensor.
pub struct TemperatureSensor {
    name: String,
    /// Path to the temperature file (millidegrees Celsius)
    temp_path: Option<PathBuf>,
    /// Last sampled temperature in Celsius
    last_temp: Option<f64>,
}

impl TemperatureSensor {
    /// Creates a new temperature sensor with auto-detection.
    pub fn new() -> Self {
        let temp_path = Self::detect_temp_path();
        if let Some(ref path) = temp_path {
            debug!("Temperature sensor using: {:?}", path);
        } else {
            debug!("No temperature sensor found");
        }

        Self {
            name: "cpu_temperature".to_string(),
            temp_path,
            last_temp: None,
        }
    }

    /// Detects the best temperature sensor path.
    /// Prefers CPU-specific sensors over generic thermal zones.
    fn detect_temp_path() -> Option<PathBuf> {
        // Try hwmon first - look for CPU-related sensors
        // Common labels: "coretemp", "k10temp", "cpu_thermal"
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let hwmon_path = entry.path();

                // Check the name of this hwmon device
                let name_path = hwmon_path.join("name");
                if let Ok(name) = fs::read_to_string(&name_path) {
                    let name = name.trim();
                    // CPU temperature sensors
                    if name == "coretemp"
                        || name == "k10temp"
                        || name == "zenpower"
                        || name == "cpu_thermal"
                        || name == "acpitz"
                    {
                        // Find temp1_input (primary temperature)
                        let temp_path = hwmon_path.join("temp1_input");
                        if temp_path.exists() {
                            return Some(temp_path);
                        }
                    }
                }
            }
        }

        // Fallback to thermal_zone - prefer zone0 (usually CPU)
        for i in 0..10 {
            let zone_path = PathBuf::from(format!("/sys/class/thermal/thermal_zone{}", i));
            if zone_path.exists() {
                // Check the type to prefer CPU zones
                let type_path = zone_path.join("type");
                if let Ok(zone_type) = fs::read_to_string(&type_path) {
                    let zone_type = zone_type.trim().to_lowercase();
                    if zone_type.contains("cpu")
                        || zone_type.contains("core")
                        || zone_type.contains("x86")
                        || zone_type == "acpitz"
                    {
                        let temp_path = zone_path.join("temp");
                        if temp_path.exists() {
                            return Some(temp_path);
                        }
                    }
                }
            }
        }

        // Last resort: just use thermal_zone0 if it exists
        let zone0_temp = PathBuf::from("/sys/class/thermal/thermal_zone0/temp");
        if zone0_temp.exists() {
            return Some(zone0_temp);
        }

        None
    }

    /// Returns the last sampled temperature in Celsius.
    pub fn temperature(&self) -> Option<f64> {
        self.last_temp
    }

    /// Reads the current temperature from the sensor.
    fn read_temp(&self) -> Option<f64> {
        let path = self.temp_path.as_ref()?;
        let content = fs::read_to_string(path).ok()?;
        let millidegrees: f64 = content.trim().parse().ok()?;
        // Temperature is in millidegrees Celsius
        Some(millidegrees / 1000.0)
    }
}

impl Default for TemperatureSensor {
    fn default() -> Self {
        Self::new()
    }
}

impl Sensor for TemperatureSensor {
    fn name(&self) -> &str {
        &self.name
    }

    fn sample(&mut self) -> f64 {
        self.last_temp = self.read_temp();
        self.last_temp.unwrap_or(0.0)
    }

    fn min(&self) -> f64 {
        0.0
    }

    fn max(&self) -> f64 {
        120.0 // Max reasonable CPU temperature
    }

    fn unit(&self) -> &str {
        "°C"
    }
}
