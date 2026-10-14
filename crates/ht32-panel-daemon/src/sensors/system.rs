//! System information sensor for hostname, uptime, and time.

use chrono::{Datelike, Local, Timelike};
use std::fs;

/// System information provider.
pub struct SystemInfo;

impl SystemInfo {
    /// Creates a new system info provider.
    pub fn new() -> Self {
        Self
    }

    /// Returns the hostname of the system.
    pub fn hostname(&self) -> String {
        fs::read_to_string("/etc/hostname")
            .map(|s| s.trim().to_string())
            .or_else(|_| {
                fs::read_to_string("/proc/sys/kernel/hostname").map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|_| "unknown".to_string())
    }

    /// Returns the current time formatted as "HH:MM".
    pub fn time(&self) -> String {
        let (hours, minutes, _, _, _, _, _) = self.time_components();
        format!("{:02}:{:02}", hours, minutes)
    }

    /// Returns individual time/date components: (hour, minute, day, month, year, day_of_week).
    /// Day of week: 0=Sunday, 1=Monday, ..., 6=Saturday.
    /// Uses the system's local timezone.
    pub fn time_components(&self) -> (u8, u8, u8, u8, u16, u8, u64) {
        let now = Local::now();

        let hours = now.hour() as u8;
        let minutes = now.minute() as u8;
        let day = now.day() as u8;
        let month = now.month() as u8;
        let year = now.year() as u16;
        // chrono: Mon=0, Tue=1, ..., Sun=6; we need Sun=0, Mon=1, ..., Sat=6
        let day_of_week = (now.weekday().num_days_from_sunday()) as u8;
        let timestamp = now.timestamp() as u64;

        (hours, minutes, day, month, year, day_of_week, timestamp)
    }

    /// Returns the system uptime formatted as "Xd Yh Zm".
    pub fn uptime(&self) -> String {
        let uptime_secs = self.uptime_seconds();

        let days = uptime_secs / 86400;
        let hours = (uptime_secs % 86400) / 3600;
        let minutes = (uptime_secs % 3600) / 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    /// Returns the uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|content| {
                content
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|f| f as u64)
            })
            .unwrap_or(0)
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}
