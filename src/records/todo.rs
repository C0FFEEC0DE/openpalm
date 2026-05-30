//! ToDo record parsing
//!
//! This module implements parsing for Palm OS ToDo DB records.
//! Based on pilot-link's todo.c

use crate::error::Result;
use crate::types::PalmDateTime;

/// ToDo priority level (1-5)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Priority(pub u8);

impl Priority {
    pub fn new(level: u8) -> Self {
        Priority(level.min(5).max(1))
    }
    
    pub fn level(&self) -> u8 {
        self.0
    }
    
    pub fn is_valid(&self) -> bool {
        (1..=5).contains(&self.0)
    }
}

/// ToDo record
#[derive(Debug, Clone)]
pub struct TodoRecord {
    /// Priority (1-5)
    pub priority: Priority,
    /// Completed
    pub complete: bool,
    /// Due date
    pub due: Option<PalmDateTime>,
    /// Indefinite (no due date set)
    pub indefinite: bool,
    /// Description
    pub description: String,
    /// Note
    pub note: Option<String>,
}

impl Default for TodoRecord {
    fn default() -> Self {
        Self {
            priority: Priority(2), // Default priority
            complete: false,
            due: None,
            indefinite: true,
            description: String::new(),
            note: None,
        }
    }
}

impl TodoRecord {
    /// Create a new empty ToDo record
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create with description
    pub fn with_description(description: &str) -> Self {
        Self {
            description: description.to_string(),
            ..Default::default()
        }
    }
    
    /// Unpack from record data (todo_v1 format)
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < 3 {
            return Err(crate::error::PilotError::DlpBufSize);
        }
        
        let mut record = Self::default();
        
        // Parse due date
        let due_short = u16::from_be_bytes([data[0], data[1]]);
        
        if due_short != 0xFFFF {
            // Parse date from 16-bit Palm format
            // Format: YYYYYYYMMMMMDDDDD where Y=year-4, M=month-1, D=day
            let year = ((due_short >> 9) & 0x7F) as i32 + 4;
            let month = ((due_short >> 5) & 0x0F) as i32;
            let day = (due_short & 0x1F) as i32;
            
            let mut dt = PalmDateTime::default();
            dt.set_date(year as u16, month as u8, day as u8);
            record.due = Some(dt);
            record.indefinite = false;
        } else {
            record.indefinite = true;
        }
        
        // Parse priority
        let priority_byte = data[2];
        if (priority_byte & 0x80) != 0 {
            record.complete = true;
        }
        record.priority = Priority(priority_byte & 0x7F);
        
        // Parse description (required)
        if data.len() < 4 {
            return Err(crate::error::PilotError::DlpBufSize);
        }
        
        let desc_end = data[3..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| 3 + p)
            .unwrap_or(data.len());
        
        record.description = String::from_utf8_lossy(&data[3..desc_end]).to_string();
        
        // Parse note (optional)
        let note_start = desc_end + 1;
        if note_start < data.len() {
            let note_end = data[note_start..]
                .iter()
                .position(|&b| b == 0)
                .map(|p| note_start + p)
                .unwrap_or(data.len());
            
            if note_start < note_end {
                record.note = Some(String::from_utf8_lossy(&data[note_start..note_end]).to_string());
            }
        }
        
        Ok(record)
    }
    
    /// Pack to record data (todo_v1 format)
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Due date (2 bytes)
        if self.indefinite || self.due.is_none() {
            data.push(0xFF);
            data.push(0xFF);
        } else if let Some(ref due) = self.due {
            let (year, month, day) = due.get_date();
            // Format: YYYYYYYMMMMMDDDDD
            let due_short = (((year - 4) & 0x7F) << 9) |
                           (((month as u16 + 1) & 0x0F) << 5) |
                           ((day as u16) & 0x1F);
            data.extend_from_slice(&due_short.to_be_bytes());
        } else {
            data.push(0);
            data.push(0);
        }
        
        // Priority byte
        let mut priority_byte = self.priority.0 & 0x7F;
        if self.complete {
            priority_byte |= 0x80;
        }
        data.push(priority_byte);
        
        // Description
        data.extend_from_slice(self.description.as_bytes());
        data.push(0);
        
        // Note
        if let Some(ref note) = self.note {
            data.extend_from_slice(note.as_bytes());
            data.push(0);
        }
        
        data
    }
    
    /// Check if overdue
    pub fn is_overdue(&self) -> bool {
        if self.indefinite || self.complete {
            return false;
        }
        
        if let Some(ref due) = self.due {
            let now = PalmDateTime::now();
            return *due < now;
        }
        
        false
    }
    
    /// Get due date as string
    pub fn due_as_str(&self) -> String {
        if self.indefinite {
            return "No due date".to_string();
        }
        
        if let Some(ref due) = self.due {
            return due.format("%Y-%m-%d");
        }
        
        "Unknown".to_string()
    }
    
    /// Get status string
    pub fn status_str(&self) -> &'static str {
        if self.complete {
            "Completed"
        } else if self.is_overdue() {
            "Overdue"
        } else {
            "Pending"
        }
    }
}

/// ToDo app info
#[derive(Debug, Clone, Default)]
pub struct TodoAppInfo {
    /// Category data
    pub categories: Vec<crate::database::Category>,
    /// Last unique ID
    pub last_unique_id: u16,
    /// Number of reminders
    pub num_reminders: u8,
    /// Show completed items
    pub show_completed: bool,
    /// Sort by priority
    pub sort_by_priority: bool,
    /// Sort by due date
    pub sort_by_due_date: bool,
}

impl TodoAppInfo {
    /// Parse from app info data
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 277 {
            return Err(crate::error::PilotError::DlpBufSize);
        }

        let (categories, last_uniq_id, rest) = crate::database::parse_categories(data)?;
        let flags = rest[1];

        Ok(Self {
            categories,
            last_unique_id: last_uniq_id as u16,
            num_reminders: rest[0],
            show_completed: (flags & 0x01) != 0,
            sort_by_priority: (flags & 0x02) != 0,
            sort_by_due_date: (flags & 0x04) != 0,
        })
    }
    
    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.last_unique_id.to_be_bytes());
        data.push(self.num_reminders);
        
        let mut flags: u8 = 0;
        if self.show_completed { flags |= 0x01; }
        if self.sort_by_priority { flags |= 0x02; }
        if self.sort_by_due_date { flags |= 0x04; }
        data.push(flags);
        
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_record_new() {
        let record = TodoRecord::new();
        assert_eq!(record.priority.level(), 2);
        assert!(!record.complete);
        assert!(record.indefinite);
    }

    #[test]
    fn test_todo_record_pack_unpack() {
        let mut record = TodoRecord::new();
        record.description = "Buy groceries".to_string();
        record.note = Some("Don't forget milk!".to_string());
        record.priority = Priority(3);
        record.complete = true;
        
        let packed = record.pack();
        let unpacked = TodoRecord::unpack(&packed).unwrap();
        
        assert_eq!(unpacked.description, "Buy groceries");
        assert_eq!(unpacked.note, Some("Don't forget milk!".to_string()));
        assert_eq!(unpacked.priority.level(), 3);
        assert!(unpacked.complete);
    }

    #[test]
    fn test_priority() {
        assert!(Priority::new(0).level() == 1);
        assert!(Priority::new(5).level() == 5);
        assert!(Priority::new(10).level() == 5);
    }
}
