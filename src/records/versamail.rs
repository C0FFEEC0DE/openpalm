//! VersaMail record types for Palm OS
//!
//! This module provides VersaMail email record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// VersaMail record (extended email)
#[derive(Debug, Clone)]
pub struct VersaMailRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: VersaMailAttributes,
    /// From address
    pub from: String,
    /// Reply to
    pub reply_to: String,
    /// To addresses
    pub to: Vec<String>,
    /// CC addresses
    pub cc: Vec<String>,
    /// BCC addresses
    pub bcc: Vec<String>,
    /// Subject
    pub subject: String,
    /// Body
    pub body: String,
    /// Date sent
    pub date_sent: PalmDateTime,
    /// Date received
    pub date_received: PalmDateTime,
    /// Priority
    pub priority: MailPriority,
    /// Sensitivity
    pub sensitivity: Sensitivity,
    /// Read status
    pub is_read: bool,
    /// Has attachment
    pub has_attachment: bool,
    /// Attachment count
    pub attachment_count: u8,
    /// Folder
    pub folder: VersaMailFolder,
    /// Server UID
    pub server_uid: String,
    /// Size (bytes)
    pub size: u32,
    /// IMAP ID
    pub imap_id: u32,
}

/// VersaMail attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersaMailAttributes(u8);

impl VersaMailAttributes {
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
    Urgent = 3,
}

impl MailPriority {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => MailPriority::Low,
            2 => MailPriority::High,
            3 => MailPriority::Urgent,
            _ => MailPriority::Normal,
        }
    }
}

/// Sensitivity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Sensitivity {
    Normal = 0,
    Personal = 1,
    Private = 2,
    Confidential = 3,
}

impl Sensitivity {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => Sensitivity::Personal,
            2 => Sensitivity::Private,
            3 => Sensitivity::Confidential,
            _ => Sensitivity::Normal,
        }
    }
}

/// VersaMail folders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersaMailFolder(u8);

impl VersaMailFolder {
    pub const Inbox: Self = VersaMailFolder(0);
    pub const Outbox: Self = VersaMailFolder(1);
    pub const Drafts: Self = VersaMailFolder(2);
    pub const Sent: Self = VersaMailFolder(3);
    pub const Trash: Self = VersaMailFolder(4);
    pub const Archive: Self = VersaMailFolder(5);
    pub const Spam: Self = VersaMailFolder(6);
    
    pub fn from_u8(val: u8) -> Self {
        VersaMailFolder(val)
    }
    
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

/// Attachment info
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Filename
    pub filename: String,
    /// MIME type
    pub mime_type: String,
    /// Size in bytes
    pub size: u32,
    /// Encoding
    pub encoding: String,
    /// Part ID
    pub part_id: String,
}

impl Default for VersaMailRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: VersaMailAttributes(0),
            from: String::new(),
            reply_to: String::new(),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            body: String::new(),
            date_sent: PalmDateTime::now(),
            date_received: PalmDateTime::now(),
            priority: MailPriority::Normal,
            sensitivity: Sensitivity::Normal,
            is_read: false,
            has_attachment: false,
            attachment_count: 0,
            folder: VersaMailFolder(0),
            server_uid: String::new(),
            size: 0,
            imap_id: 0,
        }
    }
}

impl VersaMailRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 30 {
            return Err(PilotError::InvalidData("VersaMail record too short".into()));
        }

        let mut record = VersaMailRecord::default();
        let mut offset = 0;

        // Parse flags
        let flags = data[offset];
        record.is_read = (flags & 0x01) == 0;
        record.has_attachment = (flags & 0x02) != 0;
        offset += 1;

        // Priority
        record.priority = MailPriority::from_u8(data[offset]);
        offset += 1;

        // Sensitivity
        record.sensitivity = Sensitivity::from_u8(data[offset]);
        offset += 1;

        // Folder
        record.folder = VersaMailFolder::from_u8(data[offset]);
        offset += 1;

        // Attachment count
        record.attachment_count = data[offset];
        offset += 1;

        // Skip some bytes
        offset += 3;

        // Size
        record.size = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Date received
        let date_val = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.date_received = PalmDateTime::from_palm(date_val);

        // Date sent
        let date_val = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.date_sent = PalmDateTime::from_palm(date_val);

        // IMAP ID
        record.imap_id = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Parse addresses
        let (from, new_offset) = Self::parse_string(data, offset)?;
        record.from = from;
        offset = new_offset;

        let (reply_to, new_offset) = Self::parse_string(data, offset)?;
        record.reply_to = reply_to;
        offset = new_offset;

        let (to_list, new_offset) = Self::parse_string(data, offset)?;
        record.to = Self::split_addresses(&to_list);
        offset = new_offset;

        let (cc_list, new_offset) = Self::parse_string(data, offset)?;
        record.cc = Self::split_addresses(&cc_list);
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

        // Sensitivity
        data.push(self.sensitivity as u8);

        // Folder
        data.push(self.folder.as_u8());

        // Attachment count
        data.push(self.attachment_count);

        // Reserved
        data.push(0);
        data.push(0);
        data.push(0);

        // Size
        data.extend_from_slice(&self.size.to_be_bytes());

        // Date received
        data.extend_from_slice(&self.date_received.to_palm().to_be_bytes());

        // Date sent
        data.extend_from_slice(&self.date_sent.to_palm().to_be_bytes());

        // IMAP ID
        data.extend_from_slice(&self.imap_id.to_be_bytes());

        // Addresses
        data.extend_from_slice(&Self::pack_string(&self.from));
        data.extend_from_slice(&Self::pack_string(&self.reply_to));
        data.extend_from_slice(&Self::pack_string(&Self::join_addresses(&self.to)));
        data.extend_from_slice(&Self::pack_string(&Self::join_addresses(&self.cc)));
        data.extend_from_slice(&Self::pack_string(&self.subject));
        data.extend_from_slice(&Self::pack_string(&self.body));

        data
    }

    fn parse_string(data: &[u8], offset: usize) -> Result<(String, usize)> {
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

    fn split_addresses(s: &str) -> Vec<String> {
        s.split(',')
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty())
            .collect()
    }

    fn join_addresses(addrs: &[String]) -> String {
        addrs.join(",")
    }

    /// Get all recipients
    pub fn all_recipients(&self) -> Vec<String> {
        let mut all = self.to.clone();
        all.extend(self.cc.clone());
        all.extend(self.bcc.clone());
        all
    }

    /// Get body preview
    pub fn preview(&self, max_len: usize) -> String {
        if self.body.len() <= max_len {
            self.body.clone()
        } else {
            format!("{}...", &self.body[..max_len])
        }
    }
}

/// VersaMail account
#[derive(Debug, Clone)]
pub struct VersaMailAccount {
    /// Account name
    pub name: String,
    /// Email address
    pub email: String,
    /// Display name
    pub display_name: String,
    /// IMAP server
    pub imap_server: String,
    /// IMAP port
    pub imap_port: u16,
    /// IMAP use SSL
    pub imap_ssl: bool,
    /// SMTP server
    pub smtp_server: String,
    /// SMTP port
    pub smtp_port: u16,
    /// SMTP use SSL
    pub smtp_ssl: bool,
    /// Sync count
    pub sync_count: u8,
}

/// VersaMail constants
pub mod constants {
    use crate::types::FourCharCode;

    /// VersaMail database type
    pub const VERSA_MAIL_TYPE: FourCharCode = FourCharCode(0x566D6C6D); // "Vmlm"
    
    /// VersaMail database creator
    pub const VERSA_MAIL_CREATOR: FourCharCode = FourCharCode(0x566D6C6D); // "Vmlm"

    /// Default IMAP port
    pub const DEFAULT_IMAP_PORT: u16 = 993;
    
    /// Default SMTP port
    pub const DEFAULT_SMTP_PORT: u16 = 465;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_priority() {
        assert_eq!(MailPriority::from_u8(0), MailPriority::Low);
        assert_eq!(MailPriority::from_u8(1), MailPriority::Normal);
        assert_eq!(MailPriority::from_u8(3), MailPriority::Urgent);
    }

    #[test]
    fn test_sensitivity() {
        assert_eq!(Sensitivity::from_u8(0), Sensitivity::Normal);
        assert_eq!(Sensitivity::from_u8(2), Sensitivity::Private);
    }

    #[test]
    fn test_versa_mail_folder() {
        assert_eq!(VersaMailFolder::from_u8(0).as_u8(), 0);
        assert_eq!(VersaMailFolder::from_u8(6).as_u8(), 6);
        assert_eq!(VersaMailFolder::from_u8(10).as_u8(), 10);
    }

    #[test]
    fn test_split_addresses() {
        let addr = "a@b.com, c@d.com, e@f.com";
        let split = VersaMailRecord::split_addresses(addr);
        assert_eq!(split.len(), 3);
        assert_eq!(split[0], "a@b.com");
    }

    #[test]
    fn test_versa_mail_record_pack_parse() {
        let mut record = VersaMailRecord::default();
        record.from = "sender@example.com".to_string();
        record.to = vec!["recipient@example.com".to_string()];
        record.subject = "Test subject".to_string();
        record.body = "Test body content".to_string();
        record.is_read = true;
        record.priority = MailPriority::High;

        let packed = record.pack();
        let parsed = VersaMailRecord::parse(&packed).unwrap();
        
        assert_eq!(parsed.from, "sender@example.com");
        assert_eq!(parsed.subject, "Test subject");
        assert!(parsed.is_read);
        assert_eq!(parsed.priority, MailPriority::High);
    }

    #[test]
    fn test_preview() {
        let record = VersaMailRecord {
            body: "Short".to_string(),
            ..Default::default()
        };
        assert_eq!(record.preview(10), "Short");

        let long_record = VersaMailRecord {
            body: "This is a very long body".to_string(),
            ..Default::default()
        };
        assert!(long_record.preview(10).ends_with("..."));
    }
}
