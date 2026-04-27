//! Memo record parsing
//!
//! This module implements parsing for Palm OS Memo DB records.
//! Based on pilot-link's memo.c

use crate::error::Result;

/// Memo record (simple text record)
#[derive(Debug, Clone)]
pub struct MemoRecord {
    /// Memo text
    pub text: String,
}

impl Default for MemoRecord {
    fn default() -> Self {
        Self {
            text: String::new(),
        }
    }
}

impl MemoRecord {
    /// Create a new empty memo
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create with text
    pub fn with_text(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
    
    /// Unpack from record data (memo_v1 format)
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(crate::error::PilotError::DlpBufSize);
        }
        
        // Memo is just a null-terminated string
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        let text = String::from_utf8_lossy(&data[..end]).to_string();
        
        Ok(Self { text })
    }
    
    /// Pack to record data (memo_v1 format)
    pub fn pack(&self) -> Vec<u8> {
        let mut data = self.text.as_bytes().to_vec();
        data.push(0);
        data
    }
    
    /// Get word count
    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
    
    /// Get line count
    pub fn line_count(&self) -> usize {
        self.text.lines().count()
    }
    
    /// Get character count
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }
}

/// Memo app info
#[derive(Debug, Clone, Default)]
pub struct MemoAppInfo {
    /// Category data
    pub categories: Vec<crate::database::Category>,
    /// Last unique ID
    pub last_unique_id: u16,
}

impl MemoAppInfo {
    /// Parse from app info data
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(crate::error::PilotError::DlpBufSize);
        }
        
        let mut info = Self::default();
        info.last_unique_id = u16::from_be_bytes([data[0], data[1]]);
        
        Ok(info)
    }
    
    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.last_unique_id.to_be_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_record_new() {
        let memo = MemoRecord::new();
        assert!(memo.text.is_empty());
    }

    #[test]
    fn test_memo_record_pack_unpack() {
        let memo = MemoRecord::with_text("This is a test memo.");
        
        let packed = memo.pack();
        let unpacked = MemoRecord::unpack(&packed).unwrap();
        
        assert_eq!(unpacked.text, "This is a test memo.");
    }

    #[test]
    fn test_word_count() {
        let memo = MemoRecord::with_text("One two three four five");
        assert_eq!(memo.word_count(), 5);
    }
}
