//! Notepad record types for Palm OS
//!
//! This module provides notepad/note record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// Notepad record
#[derive(Debug, Clone)]
pub struct NotepadRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: NotepadAttributes,
    /// Created date
    pub created: PalmDateTime,
    /// Note text (handwriting)
    pub note_text: String,
    /// Binary data (stroke data)
    pub stroke_data: Vec<u8>,
}

/// Notepad attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotepadAttributes(u8);

impl NotepadAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;

    pub fn is_secret(&self) -> bool {
        (self.0 & Self::SECRET) != 0
    }
    pub fn is_busy(&self) -> bool {
        (self.0 & Self::BUSY) != 0
    }
    pub fn is_archived(&self) -> bool {
        (self.0 & Self::ARCHIVE) != 0
    }
}

impl Default for NotepadRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: NotepadAttributes(0),
            created: PalmDateTime::now(),
            note_text: String::new(),
            stroke_data: Vec::new(),
        }
    }
}

impl NotepadRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 6 {
            return Err(PilotError::InvalidData("Notepad record too short".into()));
        }

        let mut offset = 0;

        // Created date (Palm timestamp)
        let created_val = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let created = PalmDateTime::from_palm(created_val);

        // Unique ID
        let unique_id = u16::from_be_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Parse text portion
        let text_len = if data.len() > offset {
            let len = data[offset] as usize;
            if offset + len + 1 > data.len() {
                data.len() - offset - 1
            } else {
                len
            }
        } else {
            0
        };

        let note_text = if text_len > 0 && offset + 1 + text_len <= data.len() {
            crate::utils::decode_palm_string(&data[offset + 1..offset + 1 + text_len])
        } else {
            String::new()
        };

        Ok(Self {
            id: unique_id as u32,
            category: 0,
            attributes: NotepadAttributes(0),
            created,
            note_text,
            stroke_data: Vec::new(),
        })
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Created date
        data.extend_from_slice(&self.created.to_palm().to_be_bytes());

        // Unique ID (truncated)
        data.extend_from_slice(&(self.id as u16).to_be_bytes());

        // Note text (Palm Notepad uses a single byte length, max 255)
        let text_bytes = crate::utils::encode_palm_string(&self.note_text);
        let truncated_len = std::cmp::min(text_bytes.len(), 255);
        data.push(truncated_len as u8);
        data.extend_from_slice(&text_bytes[..truncated_len]);

        data
    }

    /// Get text length
    pub fn text_length(&self) -> usize {
        self.note_text.len()
    }

    /// Check if has stroke data
    pub fn has_stroke_data(&self) -> bool {
        !self.stroke_data.is_empty()
    }
}

/// Notepad application info
#[derive(Debug, Clone)]
pub struct NotepadAppInfo {
    /// Categories
    pub categories: Vec<String>,
    /// Default category
    pub default_category: u8,
    /// Version
    pub version: u16,
}

impl Default for NotepadAppInfo {
    fn default() -> Self {
        Self {
            categories: vec!["Unfiled".to_string()],
            default_category: 0,
            version: 1,
        }
    }
}

/// Note types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NoteType {
    Text = 0,
    Handwriting = 1,
    Mixed = 2,
}

impl NoteType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => NoteType::Text,
            1 => NoteType::Handwriting,
            _ => NoteType::Mixed,
        }
    }
}

/// Notepad constants
pub mod constants {
    use crate::types::FourCharCode;

    /// Notepad database type
    pub const NOTEPAD_TYPE: FourCharCode = FourCharCode(0x4E6F7465);

    /// Notepad database creator
    pub const NOTEPAD_CREATOR: FourCharCode = FourCharCode(0x4E6F7465);

    /// Maximum note length
    pub const MAX_NOTE_LENGTH: usize = 4096;

    /// Maximum stroke count
    pub const MAX_STROKES: usize = 1000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notepad_attributes() {
        let attrs = NotepadAttributes(NotepadAttributes::SECRET | NotepadAttributes::BUSY);
        assert!(attrs.is_secret());
        assert!(attrs.is_busy());
        assert!(!attrs.is_archived());
    }

    #[test]
    fn test_notepad_record_pack_parse() {
        let mut record = NotepadRecord::default();
        record.note_text = "Test note".to_string();

        let packed = record.pack();
        let parsed = NotepadRecord::parse(&packed).unwrap();

        assert_eq!(parsed.note_text, "Test note");
    }

    #[test]
    fn test_note_type() {
        assert_eq!(NoteType::from_u8(0), NoteType::Text);
        assert_eq!(NoteType::from_u8(1), NoteType::Handwriting);
        assert_eq!(NoteType::from_u8(2), NoteType::Mixed);
    }
}
