//! Palm OS date/time conversion utilities

use std::time::SystemTime;

/// Number of seconds between Palm epoch (Jan 1, 1904) and Unix epoch (Jan 1, 1970)
pub const PALM_EPOCH_TO_UNIX_EPOCH: i64 = 2082844800;

/// Palm OS undefined date value
pub const PALM_UNDEFINED_DATE: u32 = 0x83DAC000;

/// Convert Unix time_t to Palm OS date/time
#[inline]
pub fn to_palm_time(unix_time: i64) -> u32 {
    (unix_time as i64 + PALM_EPOCH_TO_UNIX_EPOCH) as u32
}

/// Convert Palm OS date/time to Unix time_t
#[inline]
pub fn from_palm_time(palm_time: u32) -> i64 {
    (palm_time as i64) - PALM_EPOCH_TO_UNIX_EPOCH
}

/// Convert Palm OS date/time to Unix SystemTime
#[inline]
pub fn palm_to_system_time(palm_time: u32) -> SystemTime {
    let unix_secs = from_palm_time(palm_time);
    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(unix_secs as u64)
}

/// Convert Unix SystemTime to Palm OS date/time
#[inline]
pub fn system_time_to_palm(time: SystemTime) -> u32 {
    let duration = time.duration_since(SystemTime::UNIX_EPOCH)
        .expect("time before Unix epoch");
    let unix_secs = duration.as_secs() as i64;
    to_palm_time(unix_secs)
}

/// A Palm OS date/time value
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PalmDateTime(u32);

impl PalmDateTime {
    /// Create a new PalmDateTime from a Palm timestamp
    pub fn from_palm(raw: u32) -> Self {
        PalmDateTime(raw)
    }

    /// Get the raw Palm timestamp
    pub fn to_palm(&self) -> u32 {
        self.0
    }

    /// Alias for from_palm (for API compatibility)
    pub fn from_palm_time(raw: u32) -> Self {
        Self::from_palm(raw)
    }

    /// Alias for to_palm (for API compatibility)
    pub fn to_palm_time(&self) -> u32 {
        self.to_palm()
    }

    /// Create from Unix timestamp
    pub fn from_unix(unix_secs: i64) -> Self {
        PalmDateTime(to_palm_time(unix_secs))
    }

    /// Convert to Unix timestamp
    pub fn to_unix(&self) -> i64 {
        from_palm_time(self.0)
    }

    /// Check if this is the undefined date
    pub fn is_undefined(&self) -> bool {
        self.0 == PALM_UNDEFINED_DATE
    }

    /// Create an undefined date
    pub fn undefined() -> Self {
        PalmDateTime(PALM_UNDEFINED_DATE)
    }

    /// Set date components (year, month, day)
    pub fn set_date(&mut self, year: u16, month: u8, day: u8) {
        // This creates a Palm timestamp from components
        // For simplicity, using a placeholder - real implementation would convert
        // based on Palm epoch
        let _ = (year, month, day);
    }

    /// Get date components (year, month, day)
    pub fn get_date(&self) -> (u16, u8, u8) {
        // Simplified - returns (year, month, day) from Palm timestamp
        // This is a placeholder for proper conversion
        if self.is_undefined() {
            return (0, 0, 0);
        }
        // Convert Palm time to Unix and then extract date components
        let unix_secs = self.to_unix() as i64;
        let days = unix_secs / 86400;
        // Approximate calculation from Unix epoch
        let mut year: i64 = 1970;
        let mut remaining_days = days;
        while remaining_days >= 365 {
            let leap = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
            if remaining_days >= leap {
                remaining_days -= leap;
                year += 1;
            } else {
                break;
            }
        }
        let mut month: i64 = 1;
        let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for (i, &days_in_month) in days_in_months.iter().enumerate() {
            if remaining_days < days_in_month as i64 {
                break;
            }
            remaining_days -= days_in_month as i64;
            month = i as i64 + 1;
        }
        let day = remaining_days + 1;
        (year as u16, month as u8, day as u8)
    }

    /// Get current time
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self::from_unix(duration.as_secs() as i64)
    }

    /// Format as string
    pub fn format(&self, fmt: &str) -> String {
        let (year, month, day) = self.get_date();
        fmt.replace("%Y", &year.to_string())
           .replace("%m", &format!("{:02}", month))
           .replace("%d", &format!("{:02}", day))
    }
}

impl Default for PalmDateTime {
    fn default() -> Self {
        Self::undefined()
    }
}

impl From<u32> for PalmDateTime {
    fn from(raw: u32) -> Self {
        Self::from_palm(raw)
    }
}

impl From<PalmDateTime> for u32 {
    fn from(dt: PalmDateTime) -> Self {
        dt.to_palm()
    }
}

/// Parse Palm OS date/time from 4-byte little-endian format
pub fn parse_palm_date_le(bytes: &[u8]) -> Result<u32, &'static str> {
    if bytes.len() < 4 {
        return Err("Buffer too small for Palm date");
    }
    Ok(u32::from_le_bytes(
        bytes[0..4].try_into().map_err(|_| "Invalid buffer")?
    ))
}

/// Write Palm OS date/time to 4-byte little-endian format
pub fn write_palm_date_le(value: u32, bytes: &mut [u8]) -> Result<(), &'static str> {
    if bytes.len() < 4 {
        return Err("Buffer too small for Palm date");
    }
    bytes[0..4].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

/// Check if a year is a leap year
pub fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get the number of days in a month
pub fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_offset() {
        // to_palm_time(0) should return the number of seconds from Palm epoch to Unix epoch
        // Unix epoch (Jan 1, 1970) - Palm epoch (Jan 1, 1904) = 66 years = 2082844800 seconds
        // 0x83DAC000 is the "undefined" date marker, not a computed offset
        let palm = to_palm_time(0);
        assert_eq!(palm, 2082844800); // = 0x7C268C00
    }

    #[test]
    fn test_roundtrip() {
        let original = 1234567890i64;
        let palm = to_palm_time(original);
        let restored = from_palm_time(palm);
        assert_eq!(original, restored);
    }

    #[test]
    fn test_palm_datetime() {
        let dt = PalmDateTime::from_palm(0x83DAC000);
        assert!(dt.is_undefined());
    }

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
    }
}
