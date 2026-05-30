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
        if offset + 6 <= data.len() {
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
        if offset + 2 <= data.len() {
            let minutes = u16::from_be_bytes([data[offset], data[offset + 1]]);
            if minutes > 0 {
                record.alarm = Some(AlarmInfo {
                    minutes,
                    unit: 0,
                    sound_repeat: 0,
                });
            }
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
            data.extend_from_slice(&alarm.minutes.to_be_bytes());
        }

        data
    }

    fn parse_string(data: &[u8], offset: usize) -> Result<(String, usize)> {
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        let s = String::from_utf8_lossy(&data[offset..end]).to_string();
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
}
