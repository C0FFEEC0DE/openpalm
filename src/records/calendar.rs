//! Calendar record parsing
//!
//! This module implements parsing for Palm OS Calendar DB records.
//! Based on pilot-link's calendar.c

use crate::error::Result;
use crate::types::PalmDateTime;

/// Alarm units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlarmUnit {
    Minutes = 0,
    Hours = 1,
    Days = 2,
}

/// Calendar record flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarFlags(pub u8);

impl CalendarFlags {
    pub const ALARM: u8 = 64;
    pub const REPEAT: u8 = 32;
    pub const NOTE: u8 = 16;
    pub const EXCEPTIONS: u8 = 8;
    pub const DESCRIPTION: u8 = 4;
    pub const LOCATION: u8 = 2;
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

/// Calendar event
#[derive(Debug, Clone)]
pub struct CalendarEvent {
    /// Event ID
    pub event: u32,
    /// Start time
    pub begin: PalmDateTime,
    /// End time
    pub end: PalmDateTime,
    /// Alarm minutes before event
    pub alarm: u16,
    /// Advance time for alarm
    pub advance: u8,
    /// Advance units
    pub advance_units: AlarmUnit,
    /// Repeat type
    pub repeat_type: RepeatType,
    /// Repeat forever
    pub repeat_forever: bool,
    /// Repeat end date
    pub repeat_end: PalmDateTime,
    /// Repeat frequency
    pub repeat_frequency: u16,
    /// Repeat day (for monthly)
    pub repeat_day: u8,
    /// Repeat days (for weekly, bitmask)
    pub repeat_days: [bool; 7],
    /// Repeat week start
    pub repeat_weekstart: u8,
    /// Number of exceptions
    pub exceptions: u16,
    /// Exception dates
    pub exception: Vec<PalmDateTime>,
    /// Description
    pub description: Option<String>,
    /// Note
    pub note: Option<String>,
    /// Location
    pub location: Option<String>,
}

impl Default for CalendarEvent {
    fn default() -> Self {
        Self {
            event: 0,
            begin: PalmDateTime::default(),
            end: PalmDateTime::default(),
            alarm: 0,
            advance: 0,
            advance_units: AlarmUnit::Minutes,
            repeat_type: RepeatType::None,
            repeat_forever: false,
            repeat_end: PalmDateTime::default(),
            repeat_frequency: 0,
            repeat_day: 0,
            repeat_days: [false; 7],
            repeat_weekstart: 0,
            exceptions: 0,
            exception: Vec::new(),
            description: None,
            note: None,
            location: None,
        }
    }
}

impl CalendarEvent {
    /// Create a new empty event
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if has alarm
    pub fn has_alarm(&self) -> bool {
        (self.alarm & 0x80) != 0
    }

    /// Get alarm minutes
    pub fn alarm_minutes(&self) -> u8 {
        self.alarm as u8 & 0x7F
    }

    /// Unpack from record data
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < 22 {
            return Err(crate::error::PilotError::DlpBufSize);
        }

        let mut event = Self::default();

        // Parse header
        event.event = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        // Parse start time
        let start_palm = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        event.begin = PalmDateTime::from_palm(start_palm);

        // Parse end time
        let end_palm = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        event.end = PalmDateTime::from_palm(end_palm);

        // Parse alarm
        event.alarm = data[12] as u16;

        // Parse advance
        event.advance = data[13];
        event.advance_units = match data[14] & 0x03 {
            0 => AlarmUnit::Minutes,
            1 => AlarmUnit::Hours,
            2 => AlarmUnit::Days,
            _ => AlarmUnit::Minutes,
        };

        // Parse flags
        let flags = data[15];
        let _has_alarm = (flags & CalendarFlags::ALARM) != 0;
        let has_repeat = (flags & CalendarFlags::REPEAT) != 0;
        let has_note = (flags & CalendarFlags::NOTE) != 0;
        let _has_exceptions = (flags & CalendarFlags::EXCEPTIONS) != 0;
        let has_description = (flags & CalendarFlags::DESCRIPTION) != 0;
        let has_location = (flags & CalendarFlags::LOCATION) != 0;

        // Parse repeat info if present
        if has_repeat && data.len() >= 22 {
            event.repeat_type = match data[16] {
                0 => RepeatType::None,
                1 => RepeatType::Daily,
                2 => RepeatType::Weekly,
                3 => RepeatType::MonthlyByDay,
                4 => RepeatType::MonthlyByDate,
                5 => RepeatType::Yearly,
                _ => RepeatType::None,
            };

            event.repeat_forever = (data[17] & 0x80) != 0;

            // Parse repeat frequency
            event.repeat_frequency = u16::from_be_bytes([data[18], data[19]]);

            // Parse repeat days (for weekly)
            event.repeat_days[0] = (data[20] & 0x01) != 0;
            event.repeat_days[1] = (data[20] & 0x02) != 0;
            event.repeat_days[2] = (data[20] & 0x04) != 0;
            event.repeat_days[3] = (data[20] & 0x08) != 0;
            event.repeat_days[4] = (data[20] & 0x10) != 0;
            event.repeat_days[5] = (data[20] & 0x20) != 0;
            event.repeat_days[6] = (data[20] & 0x40) != 0;

            event.repeat_day = data[21];
            event.repeat_weekstart = data[22];

            let mut repeat_offset = 23;
            if !event.repeat_forever && data.len() >= repeat_offset + 4 {
                event.repeat_end = PalmDateTime::from_palm(u32::from_be_bytes([
                    data[repeat_offset],
                    data[repeat_offset + 1],
                    data[repeat_offset + 2],
                    data[repeat_offset + 3],
                ]));
                repeat_offset += 4;
            }

            if _has_exceptions && data.len() >= repeat_offset + 2 {
                event.exceptions =
                    u16::from_be_bytes([data[repeat_offset], data[repeat_offset + 1]]);
                repeat_offset += 2;
                for _ in 0..event.exceptions {
                    if data.len() >= repeat_offset + 4 {
                        event
                            .exception
                            .push(PalmDateTime::from_palm(u32::from_be_bytes([
                                data[repeat_offset],
                                data[repeat_offset + 1],
                                data[repeat_offset + 2],
                                data[repeat_offset + 3],
                            ])));
                        repeat_offset += 4;
                    }
                }
            }
        }

        // Determine string offset after repeat/exceptions block
        let mut offset = if has_repeat {
            let base = 23usize;
            let mut end_offset = base;
            if !event.repeat_forever {
                end_offset += 4;
            }
            if _has_exceptions && data.len() >= end_offset + 2 {
                let exc_count =
                    u16::from_be_bytes([data[end_offset], data[end_offset + 1]]) as usize;
                end_offset += 2 + exc_count * 4;
            }
            end_offset
        } else {
            16
        };

        if has_description {
            let end = data[offset..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(data.len() - offset);
            event.description = Some(crate::utils::decode_palm_string(
                &data[offset..offset + end],
            ));
            offset += end + 1;
        }

        if has_note && offset < data.len() {
            let end = data[offset..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(data.len() - offset);
            event.note = Some(crate::utils::decode_palm_string(
                &data[offset..offset + end],
            ));
            offset += end + 1;
        }

        if has_location && offset < data.len() {
            let end = data[offset..]
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(data.len() - offset);
            event.location = Some(crate::utils::decode_palm_string(
                &data[offset..offset + end],
            ));
        }

        Ok(event)
    }

    /// Pack to record data
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(&self.event.to_be_bytes());
        data.extend_from_slice(&self.begin.to_palm().to_be_bytes());
        data.extend_from_slice(&self.end.to_palm().to_be_bytes());
        data.push(self.alarm as u8);
        data.push(self.advance);
        data.push(self.advance_units as u8);

        // Flags
        let mut flags: u8 = 0;
        if self.has_alarm() {
            flags |= CalendarFlags::ALARM;
        }
        if self.repeat_type != RepeatType::None {
            flags |= CalendarFlags::REPEAT;
        }
        if self.note.is_some() {
            flags |= CalendarFlags::NOTE;
        }
        if self.description.is_some() {
            flags |= CalendarFlags::DESCRIPTION;
        }
        if self.location.is_some() {
            flags |= CalendarFlags::LOCATION;
        }
        if self.exceptions > 0 {
            flags |= CalendarFlags::EXCEPTIONS;
        }
        data.push(flags);

        // Repeat info
        if self.repeat_type != RepeatType::None {
            data.push(self.repeat_type as u8);
            data.push(if self.repeat_forever { 0x80 } else { 0 });
            data.extend_from_slice(&self.repeat_frequency.to_be_bytes());

            let mut day_bits: u8 = 0;
            for (i, &day) in self.repeat_days.iter().enumerate() {
                if day {
                    day_bits |= 1 << i;
                }
            }
            data.push(day_bits);
            data.push(self.repeat_day);
            data.push(self.repeat_weekstart);

            if !self.repeat_forever {
                data.extend_from_slice(&self.repeat_end.to_palm().to_be_bytes());
            }
        }

        // Exceptions
        if self.exceptions > 0 {
            data.extend_from_slice(&self.exceptions.to_be_bytes());
            for ex in &self.exception {
                data.extend_from_slice(&ex.to_palm().to_be_bytes());
            }
        }

        // Strings
        if let Some(ref desc) = self.description {
            data.extend_from_slice(&crate::utils::encode_palm_string(desc));
            data.push(0);
        }

        if let Some(ref note) = self.note {
            data.extend_from_slice(&crate::utils::encode_palm_string(note));
            data.push(0);
        }

        if let Some(ref loc) = self.location {
            data.extend_from_slice(&crate::utils::encode_palm_string(loc));
            data.push(0);
        }

        data
    }

    /// Get repeat description
    pub fn repeat_description(&self) -> String {
        match self.repeat_type {
            RepeatType::None => "No repeat".to_string(),
            RepeatType::Daily => format!("Daily every {} day(s)", self.repeat_frequency),
            RepeatType::Weekly => format!("Weekly every {} week(s)", self.repeat_frequency),
            RepeatType::MonthlyByDay => {
                format!("Monthly by day every {} month(s)", self.repeat_frequency)
            }
            RepeatType::MonthlyByDate => {
                format!("Monthly by date every {} month(s)", self.repeat_frequency)
            }
            RepeatType::Yearly => "Yearly".to_string(),
        }
    }
}

/// Calendar app info
#[derive(Debug, Clone, Default)]
pub struct CalendarAppInfo {
    /// Category data
    pub categories: Vec<crate::database::Category>,
    /// Last unique ID
    pub last_unique_id: u16,
    /// Number of reminders
    pub num_reminders: u8,
    /// Default alarm minutes
    pub default_alarm_minutes: u8,
    /// Default view mode
    pub default_view_mode: u8,
    /// Show time bars
    pub show_time_bars: bool,
}

impl CalendarAppInfo {
    /// Parse from app info data
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 279 {
            return Err(crate::error::PilotError::DlpBufSize);
        }

        let (categories, last_uniq_id, rest) = crate::database::parse_categories(data)?;

        Ok(Self {
            categories,
            last_unique_id: last_uniq_id as u16,
            num_reminders: rest[0],
            default_alarm_minutes: rest[1],
            default_view_mode: rest[2],
            show_time_bars: rest[3] != 0,
        })
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(279);
        let mut renamed: u16 = 0;
        for (i, cat) in self.categories.iter().enumerate() {
            if cat.flags != 0 {
                renamed |= 1 << i;
            }
        }
        data.extend_from_slice(&renamed.to_be_bytes());
        for cat in &self.categories {
            data.extend_from_slice(&cat.name);
        }
        for cat in &self.categories {
            data.push(cat.reserved);
        }
        data.push(self.last_unique_id as u8);
        data.push(self.num_reminders);
        data.push(self.default_alarm_minutes);
        data.push(self.default_view_mode);
        data.push(if self.show_time_bars { 1 } else { 0 });
        data
    }
}

/// Calendar DB info
#[derive(Debug, Clone, Default)]
pub struct CalendarDbInfo {
    /// Version
    pub version: u16,
    /// Last view mode
    pub last_view_mode: u8,
    /// Last selected date (Palm date format)
    pub last_date: u32,
    /// Default calendar
    pub default_calendar: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_event_new() {
        let event = CalendarEvent::new();
        assert_eq!(event.event, 0);
        assert!(!event.has_alarm());
    }

    #[test]
    fn test_calendar_flags() {
        assert_eq!(CalendarFlags::ALARM, 64);
        assert_eq!(CalendarFlags::REPEAT, 32);
        assert_eq!(CalendarFlags::NOTE, 16);
    }

    #[test]
    fn test_repeat_description() {
        let mut event = CalendarEvent::new();
        event.repeat_type = RepeatType::Daily;
        event.repeat_frequency = 1;

        assert_eq!(event.repeat_description(), "Daily every 1 day(s)");
    }

    #[test]
    fn test_calendar_roundtrip_with_repeat_and_exceptions() {
        let mut event = CalendarEvent::new();
        event.event = 42;
        event.begin = crate::types::PalmDateTime::from_palm(0x83DAC000); // undefined
        event.end = crate::types::PalmDateTime::from_palm(0x83DAC000);
        event.repeat_type = RepeatType::Weekly;
        event.repeat_frequency = 2;
        event.repeat_days = [true, false, true, false, true, false, false];
        event.repeat_end = crate::types::PalmDateTime::from_palm(0x12345678);
        event.repeat_weekstart = 1; // Monday
        event.exceptions = 2;
        event.exception = vec![
            crate::types::PalmDateTime::from_palm(0x11111111),
            crate::types::PalmDateTime::from_palm(0x22222222),
        ];
        event.description = Some("Weekly meeting".to_string());

        let packed = event.pack();
        let unpacked = CalendarEvent::unpack(&packed).unwrap();

        assert_eq!(unpacked.event, 42);
        assert_eq!(unpacked.repeat_type, RepeatType::Weekly);
        assert_eq!(unpacked.repeat_frequency, 2);
        assert_eq!(
            unpacked.repeat_days,
            [true, false, true, false, true, false, false]
        );
        assert_eq!(unpacked.repeat_end.to_palm(), 0x12345678);
        assert_eq!(unpacked.repeat_weekstart, 1);
        assert_eq!(unpacked.exceptions, 2);
        assert_eq!(unpacked.exception.len(), 2);
        assert_eq!(unpacked.exception[0].to_palm(), 0x11111111);
        assert_eq!(unpacked.exception[1].to_palm(), 0x22222222);
        assert_eq!(unpacked.description, Some("Weekly meeting".to_string()));
    }
}
