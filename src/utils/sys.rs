//! System utilities for openpalm
//!
//! This module provides system-level utilities.

use std::time::{Duration, SystemTime};

/// Get PILOTRATE environment variable
pub fn get_pilot_rate() -> (i32, bool) {
    // Default PADP connection rate
    if let Ok(rate_env) = std::env::var("PILOTRATE") {
        if rate_env.starts_with('H') {
            let rate = rate_env[1..].parse().unwrap_or(-1);
            (rate, true)
        } else {
            let rate = rate_env.parse().unwrap_or(-1);
            (rate, false)
        }
    } else {
        (-1, false)
    }
}

/// Convert timeout in milliseconds to Duration
pub fn timeout_to_duration(timeout_ms: u64) -> Duration {
    Duration::from_millis(timeout_ms)
}

/// Convert timeout to absolute SystemTime from now
pub fn timeout_to_system_time(timeout_ms: u64) -> SystemTime {
    SystemTime::now() + Duration::from_millis(timeout_ms)
}

/// Check if a timeout has expired
pub fn timeout_expired(timeout: SystemTime) -> bool {
    SystemTime::now() > timeout
}

/// Convert SystemTime to timeout from now (in milliseconds)
pub fn system_time_to_timeout(timeout: SystemTime) -> i64 {
    let now = SystemTime::now();
    if timeout > now {
        timeout.duration_since(now)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    } else {
        -(now.duration_since(timeout)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0))
    }
}

/// Calculate Palm strftime (simplified)
pub fn palm_strftime(format: &str, tm: &std::time::Duration) -> String {
    let secs = tm.as_secs();
    let days = secs / 86400;
    let remaining = secs % 86400;
    
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    
    // Simple year calculation from Unix epoch
    let mut year = 1970i64;
    let mut remaining_days = days as i64;
    while remaining_days >= 365 {
        let leap = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining_days >= leap {
            remaining_days -= leap;
            year += 1;
        } else {
            break;
        }
    }
    
    // Day of year to month/day
    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_before_month = if is_leap {
        [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335]
    } else {
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334]
    };
    
    let month = days_before_month.iter()
        .enumerate()
        .filter(|(_, &d)| d <= remaining_days)
        .next_back()
        .map(|(i, _)| i + 1)
        .unwrap_or(1);
    
    let day = remaining_days - days_before_month[month - 1] + 1;
    
    format
        .replace("%Y", &format!("{}", year))
        .replace("%y", &format!("{:02}", year % 100))
        .replace("%m", &format!("{:02}", month))
        .replace("%d", &format!("{:02}", day))
        .replace("%H", &format!("{:02}", hours))
        .replace("%M", &format!("{:02}", minutes))
        .replace("%S", &format!("{:02}", seconds))
        .replace("%j", &format!("{:03}", remaining_days + 1))
}

/// Get current working directory
pub fn current_dir() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Check if running on big-endian platform
pub fn is_big_endian() -> bool {
    let num: u16 = 0x0001;
    let bytes = num.to_be_bytes();
    bytes[0] == 0
}

/// Get page size
pub fn page_size() -> usize {
    4096 // Typical page size, adjust if needed
}

/// Align address to page boundary
pub fn page_align(addr: usize) -> usize {
    let page = page_size();
    (addr + page - 1) & !(page - 1)
}

/// Get host byte order
pub fn host_byte_order() -> &'static str {
    if is_big_endian() {
        "big-endian"
    } else {
        "little-endian"
    }
}

/// Portable sleep function
pub fn sleep(duration: Duration) {
    std::thread::sleep(duration);
}

/// Get environment variable with default
pub fn env_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Set environment variable
pub fn set_env(key: &str, value: &str) -> bool {
    std::env::set_var(key, value);
    true
}

/// Clear environment variable
pub fn clear_env(key: &str) {
    std::env::remove_var(key);
}

/// Get number of CPU cores
pub fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// System information
#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    /// Operating system name
    pub os: String,
    /// Architecture
    pub arch: String,
    /// Number of CPUs
    pub cpus: usize,
    /// Page size
    pub page_size: usize,
}

impl SystemInfo {
    /// Get current system info
    pub fn current() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpus: num_cpus(),
            page_size: page_size(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_conversion() {
        let duration = timeout_to_duration(1000);
        assert_eq!(duration, Duration::from_secs(1));
    }

    #[test]
    fn test_system_info() {
        let info = SystemInfo::current();
        assert!(!info.os.is_empty());
        assert!(info.cpus >= 1);
    }

    #[test]
    fn test_page_align() {
        assert_eq!(page_align(0), 0);
        assert_eq!(page_align(1), 4096);
        assert_eq!(page_align(4096), 4096);
        assert_eq!(page_align(4097), 8192);
    }
}
