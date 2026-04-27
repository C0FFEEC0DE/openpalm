//! HiNote record types for Palm OS
//!
//! This module provides HiNote handwriting recognition record parsing.

use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// HiNote record (handwriting recognition)
#[derive(Debug, Clone)]
pub struct HiNoteRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: HiNoteAttributes,
    /// Created date
    pub created: PalmDateTime,
    /// Modified date
    pub modified: PalmDateTime,
    /// Stroke count
    pub stroke_count: u16,
    /// Ink data
    pub ink_data: Vec<u8>,
    /// Recognized text
    pub recognized_text: String,
    /// Confidence score
    pub confidence: u8,
    /// Language
    pub language: HiNoteLanguage,
}

/// HiNote attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HiNoteAttributes(u8);

impl HiNoteAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;

    pub fn is_secret(&self) -> bool { (self.0 & Self::SECRET) != 0 }
    pub fn is_busy(&self) -> bool { (self.0 & Self::BUSY) != 0 }
    pub fn is_archived(&self) -> bool { (self.0 & Self::ARCHIVE) != 0 }
}

/// HiNote languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HiNoteLanguage {
    English = 0,
    German = 1,
    French = 2,
    Spanish = 3,
    Italian = 4,
    Portuguese = 5,
    Dutch = 6,
    Swedish = 7,
    Norwegian = 8,
    Danish = 9,
    Finnish = 10,
    Japanese = 11,
    Chinese = 12,
    Korean = 13,
    Other = 14,
}

impl HiNoteLanguage {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => HiNoteLanguage::English,
            1 => HiNoteLanguage::German,
            2 => HiNoteLanguage::French,
            3 => HiNoteLanguage::Spanish,
            4 => HiNoteLanguage::Italian,
            5 => HiNoteLanguage::Portuguese,
            6 => HiNoteLanguage::Dutch,
            7 => HiNoteLanguage::Swedish,
            8 => HiNoteLanguage::Norwegian,
            9 => HiNoteLanguage::Danish,
            10 => HiNoteLanguage::Finnish,
            11 => HiNoteLanguage::Japanese,
            12 => HiNoteLanguage::Chinese,
            13 => HiNoteLanguage::Korean,
            _ => HiNoteLanguage::Other,
        }
    }
}

/// Stroke data for handwriting
#[derive(Debug, Clone)]
pub struct Stroke {
    /// X coordinates
    pub x: Vec<i16>,
    /// Y coordinates
    pub y: Vec<i16>,
    /// Pressure
    pub pressure: Vec<u8>,
    /// Timestamp delta (ms since last point)
    pub timestamps: Vec<u16>,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            x: Vec::new(),
            y: Vec::new(),
            pressure: Vec::new(),
            timestamps: Vec::new(),
        }
    }
}

/// Ink point
#[derive(Debug, Clone, Copy)]
pub struct InkPoint {
    pub x: i16,
    pub y: i16,
    pub pressure: u8,
    pub timestamp: u16,
}

impl InkPoint {
    /// Distance to another point
    pub fn distance_to(&self, other: &InkPoint) -> f64 {
        let dx = (other.x - self.x) as f64;
        let dy = (other.y - self.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

/// HiNote application info
#[derive(Debug, Clone)]
pub struct HiNoteAppInfo {
    /// Language
    pub language: HiNoteLanguage,
    /// Recognition mode
    pub recognition_mode: RecognitionMode,
    /// Auto-learn enabled
    pub auto_learn: bool,
    /// Timeout (ms)
    pub recognition_timeout: u16,
    /// Version
    pub version: u16,
}

impl Default for HiNoteAppInfo {
    fn default() -> Self {
        Self {
            language: HiNoteLanguage::English,
            recognition_mode: RecognitionMode::Mixed,
            auto_learn: true,
            recognition_timeout: 5000,
            version: 1,
        }
    }
}

/// Recognition modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecognitionMode {
    Letter = 0,
    Word = 1,
    Mixed = 2,
    Numerals = 3,
}

impl RecognitionMode {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => RecognitionMode::Letter,
            1 => RecognitionMode::Word,
            2 => RecognitionMode::Mixed,
            _ => RecognitionMode::Numerals,
        }
    }
}

impl Default for HiNoteRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: HiNoteAttributes(0),
            created: PalmDateTime::now(),
            modified: PalmDateTime::now(),
            stroke_count: 0,
            ink_data: Vec::new(),
            recognized_text: String::new(),
            confidence: 0,
            language: HiNoteLanguage::English,
        }
    }
}

impl HiNoteRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(PilotError::InvalidData("HiNote record too short".into()));
        }

        let mut record = HiNoteRecord::default();
        let mut offset = 0;

        // Created date
        let created_val = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.created = PalmDateTime::from_palm(created_val);

        // Modified date
        let modified_val = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.modified = PalmDateTime::from_palm(modified_val);

        // Stroke count
        record.stroke_count = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Language
        record.language = HiNoteLanguage::from_u8(data[offset]);
        offset += 1;

        // Confidence
        record.confidence = data[offset];
        offset += 1;

        // Skip some bytes
        offset += 2;

        // Parse recognized text
        let (text, new_offset) = Self::parse_string(data, offset)?;
        record.recognized_text = text;
        offset = new_offset;

        // Rest is ink data
        if offset < data.len() {
            record.ink_data = data[offset..].to_vec();
        }

        Ok(record)
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Created date
        data.extend_from_slice(&self.created.to_palm().to_le_bytes());

        // Modified date
        data.extend_from_slice(&self.modified.to_palm().to_le_bytes());

        // Stroke count
        data.extend_from_slice(&self.stroke_count.to_le_bytes());

        // Language
        data.push(self.language as u8);

        // Confidence
        data.push(self.confidence);

        // Reserved
        data.push(0);
        data.push(0);

        // Recognized text
        data.extend_from_slice(&Self::pack_string(&self.recognized_text));

        // Ink data
        data.extend_from_slice(&self.ink_data);

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

    /// Get stroke at index
    pub fn get_stroke(&self, _index: usize) -> Option<Stroke> {
        // In a full implementation, this would parse the ink data
        None
    }

    /// Get bounding box of ink
    pub fn bounding_box(&self) -> Option<(i16, i16, i16, i16)> {
        if self.ink_data.len() < 4 {
            return None;
        }
        
        // Simplified: return min/max from first 4 bytes
        let min_x = i16::from_le_bytes([self.ink_data[0], self.ink_data[1]]);
        let min_y = i16::from_le_bytes([self.ink_data[2], self.ink_data[3]]);
        Some((min_x, min_y, min_x + 100, min_y + 100))
    }
}

/// HiNote constants
pub mod constants {
    use crate::types::FourCharCode;

    /// HiNote database type
    pub const HINOTE_TYPE: FourCharCode = FourCharCode { 0: 0x48494E4F }; // "HINO"
    
    /// HiNote database creator
    pub const HINOTE_CREATOR: FourCharCode = FourCharCode { 0: 0x48494E4F }; // "HINO"

    /// Maximum strokes
    pub const MAX_STROKES: usize = 1000;
    
    /// Maximum points per stroke
    pub const MAX_POINTS_PER_STROKE: usize = 500;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hinote_language() {
        assert_eq!(HiNoteLanguage::from_u8(0), HiNoteLanguage::English);
        assert_eq!(HiNoteLanguage::from_u8(11), HiNoteLanguage::Japanese);
        assert_eq!(HiNoteLanguage::from_u8(20), HiNoteLanguage::Other);
    }

    #[test]
    fn test_recognition_mode() {
        assert_eq!(RecognitionMode::from_u8(0), RecognitionMode::Letter);
        assert_eq!(RecognitionMode::from_u8(2), RecognitionMode::Mixed);
        assert_eq!(RecognitionMode::from_u8(10), RecognitionMode::Numerals);
    }

    #[test]
    fn test_hinote_attributes() {
        let attrs = HiNoteAttributes(HiNoteAttributes::SECRET | HiNoteAttributes::BUSY);
        assert!(attrs.is_secret());
        assert!(attrs.is_busy());
        assert!(!attrs.is_archived());
    }

    #[test]
    fn test_ink_point_distance() {
        let p1 = InkPoint { x: 0, y: 0, pressure: 0, timestamp: 0 };
        let p2 = InkPoint { x: 3, y: 4, pressure: 0, timestamp: 100 };
        
        // 3-4-5 triangle
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_hinote_record_pack_parse() {
        let mut record = HiNoteRecord::default();
        record.recognized_text = "Hello".to_string();
        record.stroke_count = 5;
        record.confidence = 85;

        let packed = record.pack();
        let parsed = HiNoteRecord::parse(&packed).unwrap();
        
        assert_eq!(parsed.recognized_text, "Hello");
        assert_eq!(parsed.stroke_count, 5);
        assert_eq!(parsed.confidence, 85);
    }
}
