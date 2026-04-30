//! Palm OS date/time conversion utilities

use std::time::SystemTime;

/// Number of seconds between Palm epoch (Jan 1, 1904) and Unix epoch (Jan 1, 1970)
pub const PALM_EPOCH_TO_UNIX_EPOCH: i64 = 2082844800;

/// Palm OS undefined date value
pub const PALM_UNDEFINED_DATE: u32 = 0x83DAC000;

/// Convert Unix time_t to Palm OS date/time
#[inline]
pub fn to_palm_time(unix_time: i64) -> u32 {
    (unix_time + PALM_EPOCH_TO_UNIX_EPOCH) as u32
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
    /// 
    /// Note: Time components are set to 0:00:00
    pub fn set_date(&mut self, year: u16, month: u8, day: u8) {
        // Validate inputs
        if year < 1904 || !(1..=12).contains(&month) || day < 1 {
            return;
        }
        
        // Validate day against month
        let max_day = days_in_month(year, month);
        if day > max_day {
            return;
        }
        
        // Count days from Palm epoch (Jan 1, 1904) to given date
        let mut total_days: i64 = 0;
        
        // Days for years from 1904 to given year
        for y in 1904..year as i64 {
            total_days += if is_leap_year(y as u16) { 366 } else { 365 };
        }
        
        // Days for months before given month
        let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for m in 1..month as i64 {
            total_days += days_in_months[(m - 1) as usize] as i64;
        }
        
        // Adjust February for leap year
        if month == 2 && is_leap_year(year) {
            total_days += 1;
        }
        
        // Add days (Palm timestamp is 0-indexed, day parameter is 1-indexed)
        total_days += day as i64;
        
        // Convert to seconds (Palm timestamp is seconds since Palm epoch)
        self.0 = (total_days * 86400) as u32;
    }

    /// Get date components (year, month, day)
    pub fn get_date(&self) -> (u16, u8, u8) {
        if self.is_undefined() {
            return (0, 0, 0);
        }
        
        // Convert Palm timestamp to days since Palm epoch (Jan 1, 1904)
        let mut days = (self.0 as i64) / 86400;
        
        // Calculate year from Palm epoch
        let mut year: i64 = 1904;
        while days >= 365 {
            let leap = if is_leap_year(year as u16) { 366 } else { 365 };
            if days >= leap {
                days -= leap;
                year += 1;
            } else {
                break;
            }
        }
        
        // Days in each month (non-leap year)
        let days_in_months: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        
        // Calculate month (1-indexed)
        let mut month: i64 = 1;
        for (i, &dim) in days_in_months.iter().enumerate() {
            // Use 29 days for February in leap year
            let days_this_month = if i == 1 && is_leap_year(year as u16) { 29 } else { dim };
            
            if days < days_this_month {
                // We're in this month
                break;
            }
            days -= days_this_month;
            month = i as i64 + 2; // Next month
        }
        
        let day = (days + 1) as u8; // Convert from 0-indexed to 1-indexed
        
        (year as u16, month as u8, day)
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
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
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

    #[test]
    fn test_set_date() {
        // Verify set_date produces a valid Palm timestamp
        let undefined_val = PalmDateTime::undefined().to_palm();
        
        // Valid date should work
        let mut dt = PalmDateTime::undefined();
        dt.set_date(2024, 1, 1);
        assert_ne!(dt.to_palm(), undefined_val, "set_date should modify the value");
        
        // Invalid dates should not modify the value
        let mut dt = PalmDateTime::undefined();
        dt.set_date(2024, 13, 1); // Invalid month
        assert_eq!(dt.to_palm(), undefined_val, "Invalid month should not change value");
        
        let mut dt = PalmDateTime::undefined();
        dt.set_date(2024, 1, 32); // Invalid day
        assert_eq!(dt.to_palm(), undefined_val, "Invalid day should not change value");
        
        let mut dt = PalmDateTime::undefined();
        dt.set_date(1903, 1, 1); // Before Palm epoch
        assert_eq!(dt.to_palm(), undefined_val, "Year before 1904 should not change value");
    }
