//! Legacy Datebook record types for Palm OS
//!
//! This module provides datebook/appointment record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// Datebook record (appointment/event)
#[derive(Debug, Clone)]
pub struct DatebookRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: DatebookAttributes,
    /// Event date
    pub date: PalmDateTime,
    /// Start time
    pub start_time: u16,
    /// End time
    pub end_time: u16,
    /// Event type
    pub event_type: EventType,
    /// Description
    pub description: String,
    /// Note
    pub note: String,
    /// Repeat info
    pub repeat: Option<RepeatInfo>,
    /// Alarm info
    pub alarm: Option<AlarmInfo>,
}

/// Datebook attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatebookAttributes(u8);

impl DatebookAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;

    pub fn is_secret(&self) -> bool { (self.0 & Self::SECRET) != 0 }
    pub fn is_busy(&self) -> bool { (self.0 & Self::BUSY) != 0 }
    pub fn is_archived(&self) -> bool { (self.0 & Self::ARCHIVE) != 0 }
}

/// Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    Normal = 0,
    AllDay = 1,
    Meeting = 2,
}

impl EventType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => EventType::AllDay,
            2 => EventType::Meeting,
            _ => EventType::Normal,
        }
    }
}

/// Repeat types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RepeatType {
    None = 0,
    Daily = 1,
    Weekly = 2,
    MonthlyByDay = 3,
    MonthlyByDate = 4,
    Yearly = 5,
}

impl RepeatType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => RepeatType::Daily,
            2 => RepeatType::Weekly,
            3 => RepeatType::MonthlyByDay,
            4 => RepeatType::MonthlyByDate,
            5 => RepeatType::Yearly,
            _ => RepeatType::None,
        }
    }
}

/// Repeat information
#[derive(Debug, Clone)]
pub struct RepeatInfo {
    /// Repeat type
    pub repeat_type: RepeatType,
    /// Frequency (every N units)
    pub frequency: u8,
    /// Day of week (for weekly)
    pub day_of_week: u8,
    /// Day of month (for monthly)
    pub day_of_month: u8,
    /// End date
    pub end_date: PalmDateTime,
}

/// Alarm information
#[derive(Debug, Clone)]
pub struct AlarmInfo {
    /// Minutes before event
    pub minutes: u16,
    /// Unit type (0=minutes, 1=hours, 2=days)
    pub unit: u8,
    /// Sound repeat
    pub sound_repeat: u8,
}

impl AlarmInfo {
    /// Get total minutes before event
    pub fn total_minutes(&self) -> u16 {
        match self.unit {
            0 => self.minutes,
            1 => self.minutes * 60,
            2 => self.minutes * 60 * 24,
            _ => self.minutes,
        }
    }

    /// Pack to bytes (4 bytes: minutes[2] + unit[1] + sound_repeat[1])
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(4);
        data.extend_from_slice(&self.minutes.to_be_bytes());
        data.push(self.unit);
        data.push(self.sound_repeat);
        data
    }

    /// Parse from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(PilotError::InvalidData("AlarmInfo too short".into()));
        }
        Ok(Self {
            minutes: u16::from_be_bytes([data[0], data[1]]),
            unit: data[2],
            sound_repeat: data[3],
        })
    }
}

/// Datebook application info
#[derive(Debug, Clone)]
pub struct DatebookAppInfo {
    /// Categories
    pub categories: Vec<String>,
    /// Default category
    pub default_category: u8,
    /// Start of day
    pub start_hour: u8,
    /// End of day
    pub end_hour: u8,
    /// Version
    pub version: u16,
}

impl DatebookAppInfo {
    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(261);
        // 16 categories, 16 bytes each
        for i in 0..16 {
            let cat = self.categories.get(i).map(|s| s.as_str()).unwrap_or("");
            let mut bytes = cat.as_bytes().to_vec();
            bytes.resize(16, 0);
            data.extend_from_slice(&bytes);
        }
        data.push(self.default_category);
        data.push(self.start_hour);
        data.push(self.end_hour);
        data.extend_from_slice(&self.version.to_be_bytes());
        data
    }

    /// Parse from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 261 {
            return Err(PilotError::InvalidData("DatebookAppInfo too short".into()));
        }
        let mut categories = Vec::new();
        for i in 0..16 {
            let start = i * 16;
            let end = start + 16;
            let cat = crate::utils::decode_palm_string(&data[start..end])
                .trim_end_matches('\0')
                .to_string();
            if !cat.is_empty() {
                categories.push(cat);
            }
        }
        Ok(Self {
            categories,
            default_category: data[256],
            start_hour: data[257],
            end_hour: data[258],
            version: u16::from_be_bytes([data[259], data[260]]),
        })
    }
}

impl Default for DatebookAppInfo {
    fn default() -> Self {
        Self {
            categories: vec!["Unfiled".to_string()],
            default_category: 0,
            start_hour: 0,
            end_hour: 23,
            version: 1,
        }
    }
}

impl Default for DatebookRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: DatebookAttributes(0),
            date: PalmDateTime::now(),
            start_time: 0,
            end_time: 0,
            event_type: EventType::Normal,
            description: String::new(),
            note: String::new(),
            repeat: None,
            alarm: None,
        }
    }
}

impl DatebookRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(PilotError::InvalidData("Datebook record too short".into()));
        }

        let mut record = DatebookRecord::default();
        let mut offset = 0;

        // Parse date
        let date_val = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.date = PalmDateTime::from_palm(date_val);

        // Start time
        record.start_time = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // End time
        record.end_time = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Event type
        record.event_type = match data[offset] {
            1 => EventType::AllDay,
            2 => EventType::Meeting,
            _ => EventType::Normal,
        };
        offset += 1;

        // Skip some bytes
        offset += 2;

        // Parse description
        let (desc, new_offset) = Self::parse_string(data, offset)?;
        record.description = desc;
        offset = new_offset;

        // Parse note
        let (note, new_offset) = Self::parse_string(data, offset)?;
        record.note = note;
        offset = new_offset;

        // Check for repeat info (if there's enough data)
        if offset + 8 <= data.len() {
            let repeat_type = match data[offset] {
                1 => RepeatType::Daily,
                2 => RepeatType::Weekly,
                3 => RepeatType::MonthlyByDay,
                4 => RepeatType::MonthlyByDate,
                5 => RepeatType::Yearly,
                _ => RepeatType::None,
            };

            if repeat_type != RepeatType::None {
                let frequency = data[offset + 1];
                let day_of_week = data[offset + 2];
                let day_of_month = data[offset + 3];
                offset += 4;

                let end_date_val = u32::from_be_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                let end_date = PalmDateTime::from_palm(end_date_val);

                record.repeat = Some(RepeatInfo {
                    repeat_type,
                    frequency,
                    day_of_week,
                    day_of_month,
                    end_date,
                });
                offset += 4;
            }
        }

        // Check for alarm info
        if offset + 4 <= data.len() {
            record.alarm = Some(AlarmInfo::parse(&data[offset..offset + 4])?);
        }

        Ok(record)
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Date
        data.extend_from_slice(&self.date.to_palm().to_be_bytes());

        // Start time
        data.extend_from_slice(&self.start_time.to_be_bytes());

        // End time
        data.extend_from_slice(&self.end_time.to_be_bytes());

        // Event type
        data.push(self.event_type as u8);

        // Reserved
        data.push(0);
        data.push(0);

        // Description
        data.extend_from_slice(&Self::pack_string(&self.description));

        // Note
        data.extend_from_slice(&Self::pack_string(&self.note));

        // Repeat info
        if let Some(repeat) = &self.repeat {
            data.push(repeat.repeat_type as u8);
            data.push(repeat.frequency);
            data.push(repeat.day_of_week);
            data.push(repeat.day_of_month);
            data.extend_from_slice(&repeat.end_date.to_palm().to_be_bytes());
        }

        // Alarm info
        if let Some(alarm) = &self.alarm {
            data.extend_from_slice(&alarm.pack());
        }

        data
    }

    fn parse_string(data: &[u8], offset: usize) -> Result<(String, usize)> {
        if offset > data.len() {
            return Err(PilotError::InvalidData("parse_string offset out of bounds".into()));
        }
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        let s = crate::utils::decode_palm_string(&data[offset..end]);
        Ok((s, end + 1))
    }

    fn pack_string(s: &str) -> Vec<u8> {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        bytes
    }

    /// Check if event is all day
    pub fn is_all_day(&self) -> bool {
        matches!(self.event_type, EventType::AllDay)
    }

    /// Check if event repeats
    pub fn is_repeating(&self) -> bool {
        self.repeat.is_some()
    }

    /// Get duration in minutes
    pub fn duration_minutes(&self) -> u16 {
        self.end_time.saturating_sub(self.start_time)
    }

    /// Format time for display
    pub fn format_time(time: u16) -> String {
        let hour = (time / 60) as u8;
        let minute = (time % 60) as u8;
        let pm = hour >= 12;
        let display_hour = if hour == 0 { 12 } else if hour > 12 { hour - 12 } else { hour };
        format!("{:02}:{:02} {}", display_hour, minute, if pm { "PM" } else { "AM" })
    }
}

/// Datebook constants
pub mod constants {
    use crate::types::FourCharCode;

    /// Datebook database type
    pub const DATEBOOK_TYPE: FourCharCode = FourCharCode(0x44617442); // "DatB"
    
    /// Datebook database creator
    pub const DATEBOOK_CREATOR: FourCharCode = FourCharCode(0x44617442); // "DatB"

    /// Minutes in a day
    pub const MINUTES_PER_DAY: u16 = 1440;
    
    /// Maximum event duration (minutes)
    pub const MAX_DURATION: u16 = 1440;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datebook_attributes() {
        let attrs = DatebookAttributes(DatebookAttributes::SECRET | DatebookAttributes::BUSY);
        assert!(attrs.is_secret());
        assert!(attrs.is_busy());
        assert!(!attrs.is_archived());
    }

    #[test]
    fn test_event_type() {
        assert_eq!(EventType::from_u8(0), EventType::Normal);
        assert_eq!(EventType::from_u8(1), EventType::AllDay);
        assert_eq!(EventType::from_u8(2), EventType::Meeting);
    }

    #[test]
    fn test_repeat_type() {
        assert_eq!(RepeatType::from_u8(0), RepeatType::None);
        assert_eq!(RepeatType::from_u8(1), RepeatType::Daily);
        assert_eq!(RepeatType::from_u8(5), RepeatType::Yearly);
    }

    #[test]
    fn test_alarm_info() {
        let alarm = AlarmInfo { minutes: 30, unit: 0, sound_repeat: 0 };
        assert_eq!(alarm.total_minutes(), 30);

        let alarm_hours = AlarmInfo { minutes: 1, unit: 1, sound_repeat: 0 };
        assert_eq!(alarm_hours.total_minutes(), 60);
    }

    #[test]
    fn test_alarm_info_pack_parse() {
        let alarm = AlarmInfo {
            minutes: 30,
            unit: 1,
            sound_repeat: 2,
        };
        let packed = alarm.pack();
        assert_eq!(packed.len(), 4);
        let parsed = AlarmInfo::parse(&packed).unwrap();
        assert_eq!(parsed.minutes, 30);
        assert_eq!(parsed.unit, 1);
        assert_eq!(parsed.sound_repeat, 2);
    }

    #[test]
    fn test_datebook_app_info_pack_parse() {
        let info = DatebookAppInfo {
            categories: vec!["Work".to_string(), "Home".to_string()],
            default_category: 1,
            start_hour: 8,
            end_hour: 18,
            version: 2,
        };
        let packed = info.pack();
        assert_eq!(packed.len(), 261);
        let parsed = DatebookAppInfo::parse(&packed).unwrap();
        assert_eq!(parsed.categories.len(), 2);
        assert_eq!(parsed.categories[0], "Work");
        assert_eq!(parsed.categories[1], "Home");
        assert_eq!(parsed.default_category, 1);
        assert_eq!(parsed.start_hour, 8);
        assert_eq!(parsed.end_hour, 18);
        assert_eq!(parsed.version, 2);
    }

    #[test]
    fn test_format_time() {
        assert_eq!(DatebookRecord::format_time(0), "12:00 AM");
        assert_eq!(DatebookRecord::format_time(720), "12:00 PM");
        assert_eq!(DatebookRecord::format_time(555), "09:15 AM");
    }

    #[test]
    fn test_datebook_record_pack_parse() {
        let mut record = DatebookRecord::default();
        record.description = "Team meeting".to_string();
        record.note = "Discuss project".to_string();
        record.start_time = 540; // 9:00 AM
        record.end_time = 600; // 10:00 AM
        record.event_type = EventType::Normal;

        let packed = record.pack();
        let parsed = DatebookRecord::parse(&packed).unwrap();
        
        assert_eq!(parsed.description, "Team meeting");
        assert_eq!(parsed.note, "Discuss project");
        assert_eq!(parsed.duration_minutes(), 60);
    }

    #[test]
    fn test_is_all_day() {
        let mut record = DatebookRecord::default();
        assert!(!record.is_all_day());

        record.event_type = EventType::AllDay;
        assert!(record.is_all_day());
    }

    #[test]
    fn test_unpack_repeat_bounds() {
        // Minimal record with description + note but only 6 extra bytes for repeat
        // (should not read past end)
        let mut data = vec![
            // start_time, end_time, date, flags, alarm, duration
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        data.extend_from_slice(b"desc\0");
        data.extend_from_slice(b"note\0");
        // repeat_type=1, frequency=1, dow=0, dom=1  => 4 bytes
        data.push(1); // Daily
        data.push(1); // frequency
        data.push(0); // day_of_week
        data.push(1); // day_of_month
        // Only 2 bytes of end_date (need 4)
        data.push(0);
        data.push(0);

        // Should not panic — repeat should be skipped or handled gracefully
        let result = DatebookRecord::parse(&data);
        // Currently this will likely fail with an error; we just want no panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_unpack_parse_string_bounds() {
        // Description without null terminator at end of buffer
        let mut data = vec![
            // start_time, end_time, date, flags, alarm, duration
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        data.extend_from_slice(b"no-null"); // no trailing zero

        // Should not panic
        let result = DatebookRecord::parse(&data);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_string_offset_beyond_len() {
        let data = b"hello";
        // parse_string at offset > len must return an error, not panic
        let result = DatebookRecord::parse_string(data, 6);
        assert!(result.is_err());
    }
}
