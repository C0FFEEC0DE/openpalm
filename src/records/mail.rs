//! Mail record types for Palm OS
//!
//! This module provides email message record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::{FourCharCode, PalmDateTime};

/// Mail record (simplified)
#[derive(Debug, Clone)]
pub struct MailRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: MailAttributes,
    /// From address
    pub from: String,
    /// To addresses
    pub to: Vec<String>,
    /// CC addresses
    pub cc: Vec<String>,
    /// BCC addresses
    pub bcc: Vec<String>,
    /// Subject
    pub subject: String,
    /// Body preview
    pub body: String,
    /// Date sent
    pub date_sent: PalmDateTime,
    /// Date received
    pub date_received: PalmDateTime,
    /// Priority
    pub priority: MailPriority,
    /// Read status
    pub is_read: bool,
    /// Has attachment
    pub has_attachment: bool,
    /// Attachment name
    pub attachment_name: Option<String>,
    /// Folder
    pub folder: MailFolder,
}

/// Mail attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailAttributes(u8);

impl MailAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;

    pub fn is_secret(&self) -> bool { (self.0 & Self::SECRET) != 0 }
    pub fn is_busy(&self) -> bool { (self.0 & Self::BUSY) != 0 }
    pub fn is_archived(&self) -> bool { (self.0 & Self::ARCHIVE) != 0 }
}

/// Mail priority
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MailPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

impl MailPriority {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => MailPriority::Low,
            2 => MailPriority::High,
            _ => MailPriority::Normal,
        }
    }
}

/// Mail folders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailFolder(u8);

impl MailFolder {
    pub const Inbox: Self = MailFolder(0);
    pub const Outbox: Self = MailFolder(1);
    pub const Drafts: Self = MailFolder(2);
    pub const Sent: Self = MailFolder(3);
    pub const Trash: Self = MailFolder(4);
    pub const Archive: Self = MailFolder(5);
    
    pub fn from_u8(val: u8) -> Self {
        MailFolder(val)
    }
    
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    
    pub fn match_folder(&self) -> u8 {
        match self.0 {
            0..=5 => self.0,
            f => f,
        }
    }
}

impl Default for MailRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: MailAttributes(0),
            from: String::new(),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            body: String::new(),
            date_sent: PalmDateTime::now(),
            date_received: PalmDateTime::now(),
            priority: MailPriority::Normal,
            is_read: false,
            has_attachment: false,
            attachment_name: None,
            folder: MailFolder(0),
        }
    }
}

impl MailRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 20 {
            return Err(PilotError::InvalidData("Mail record too short".into()));
        }

        let mut record = MailRecord::default();
        let mut offset = 0;

        // Parse header flags
        let flags = data[offset];
        record.is_read = (flags & 0x01) == 0;
        record.has_attachment = (flags & 0x02) != 0;
        offset += 1;

        // Priority
        record.priority = MailPriority::from_u8(data[offset]);
        offset += 1;

        // Folder
        record.folder = MailFolder::from_u8(data[offset]);
        offset += 1;

        // Skip some bytes
        offset += 2;

        // Date received
        let date_val = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.date_received = PalmDateTime::from_palm(date_val);

        // Date sent
        let date_val = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.date_sent = PalmDateTime::from_palm(date_val);

        // Parse addresses
        let (from, new_offset) = Self::parse_string(data, offset)?;
        record.from = from;
        offset = new_offset;

        let (to, new_offset) = Self::parse_string_list(data, offset)?;
        record.to = to;
        offset = new_offset;

        let (cc, new_offset) = Self::parse_string_list(data, offset)?;
        record.cc = cc;
        offset = new_offset;

        let (subject, new_offset) = Self::parse_string(data, offset)?;
        record.subject = subject;
        offset = new_offset;

        let (body, _) = Self::parse_string(data, offset)?;
        record.body = body;

        Ok(record)
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Flags
        let mut flags = 0u8;
        if !self.is_read { flags |= 0x01; }
        if self.has_attachment { flags |= 0x02; }
        data.push(flags);

        // Priority
        data.push(self.priority as u8);

        // Folder
        data.push(self.folder.as_u8());

        // Reserved
        data.push(0);
        data.push(0);

        // Date received
        data.extend_from_slice(&self.date_received.to_palm().to_le_bytes());

        // Date sent
        data.extend_from_slice(&self.date_sent.to_palm().to_le_bytes());

        // Addresses
        data.extend_from_slice(&Self::pack_string(&self.from));
        data.extend_from_slice(&Self::pack_string_list(&self.to));
        data.extend_from_slice(&Self::pack_string_list(&self.cc));
        data.extend_from_slice(&Self::pack_string(&self.subject));
        data.extend_from_slice(&Self::pack_string(&self.body));

        data
    }

    /// Parse null-terminated string
    fn parse_string(data: &[u8], offset: usize) -> Result<(String, usize)> {
        let mut end = offset;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        let s = String::from_utf8_lossy(&data[offset..end]).to_string();
        Ok((s, end + 1))
    }

    /// Parse null-terminated string list (comma-separated)
    fn parse_string_list(data: &[u8], offset: usize) -> Result<(Vec<String>, usize)> {
        let (s, new_offset) = Self::parse_string(data, offset)?;
        let items = s.split(',').map(|s| s.trim().to_string()).collect();
        Ok((items, new_offset))
    }

    /// Pack string as null-terminated
    fn pack_string(s: &str) -> Vec<u8> {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        bytes
    }

    /// Pack string list
    fn pack_string_list(items: &[String]) -> Vec<u8> {
        let s = items.join(",");
        Self::pack_string(&s)
    }

    /// Get body preview (first N chars)
    pub fn body_preview(&self, max_len: usize) -> String {
        if self.body.len() <= max_len {
            self.body.clone()
        } else {
            format!("{}...", &self.body[..max_len])
        }
    }

    /// Get all recipient addresses
    pub fn all_recipients(&self) -> Vec<String> {
        let mut recipients = self.to.clone();
        recipients.extend(self.cc.clone());
        recipients.extend(self.bcc.clone());
        recipients
    }
}

/// Mail application info
#[derive(Debug, Clone)]
pub struct MailAppInfo {
    /// Accounts
    pub accounts: Vec<MailAccount>,
    /// Signatures
    pub signatures: Vec<String>,
    /// Categories
    pub categories: Vec<String>,
    /// Version
    pub version: u16,
}

/// Mail account
#[derive(Debug, Clone)]
pub struct MailAccount {
    /// Account name
    pub name: String,
    /// Email address
    pub email: String,
    /// Server (IMAP/POP)
    pub server: String,
    /// Port
    pub port: u16,
    /// Use SSL
    pub use_ssl: bool,
}

/// Mail constants
pub mod constants {
    use crate::types::FourCharCode;

    /// Mail database type
    pub const MAIL_TYPE: FourCharCode = FourCharCode { 0: 0x4D61696C };
    
    /// Mail database creator
    pub const MAIL_CREATOR: FourCharCode = FourCharCode { 0: 0x4D61696C };

    /// Maximum subject length
    pub const MAX_SUBJECT_LENGTH: usize = 256;
    
    /// Maximum body preview
    pub const MAX_BODY_PREVIEW: usize = 1024;
    
    /// Maximum attachments
    pub const MAX_ATTACHMENTS: usize = 10;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_attributes() {
        let attrs = MailAttributes(MailAttributes::SECRET | MailAttributes::ARCHIVE);
        assert!(attrs.is_secret());
        assert!(attrs.is_archived());
        assert!(!attrs.is_busy());
    }

    #[test]
    fn test_mail_priority() {
        assert_eq!(MailPriority::from_u8(0), MailPriority::Low);
        assert_eq!(MailPriority::from_u8(1), MailPriority::Normal);
        assert_eq!(MailPriority::from_u8(2), MailPriority::High);
    }

    #[test]
    fn test_mail_folder() {
        assert_eq!(MailFolder::from_u8(0), MailFolder(0));
        assert_eq!(MailFolder::from_u8(10), MailFolder(10));
        assert_eq!(MailFolder::from_u8(0).as_u8(), 0);
    }

    #[test]
    fn test_mail_record_pack_parse() {
        let mut record = MailRecord::default();
        record.from = "sender@example.com".to_string();
        record.to = vec!["recipient@example.com".to_string()];
        record.subject = "Test subject".to_string();
        record.body = "Test body".to_string();
        record.is_read = true;
        record.priority = MailPriority::High;

        let packed = record.pack();
        let parsed = MailRecord::parse(&packed).unwrap();
        
        assert_eq!(parsed.from, "sender@example.com");
        assert_eq!(parsed.subject, "Test subject");
        assert!(parsed.is_read);
        assert_eq!(parsed.priority, MailPriority::High);
    }

    #[test]
    fn test_body_preview() {
        let record = MailRecord {
            body: "Short".to_string(),
            ..Default::default()
        };
        assert_eq!(record.body_preview(10), "Short");

        let long_record = MailRecord {
            body: "This is a very long body text".to_string(),
            ..Default::default()
        };
        assert!(long_record.body_preview(10).ends_with("..."));
    }
}
