//! CMP (Communication Message Protocol) record types for Palm OS
//!
//! This module provides CMP record parsing and serialization.

use crate::error::{PilotError, Result};
use crate::types::PalmDateTime;

/// CMP record (generic communication message)
#[derive(Debug, Clone)]
pub struct CmpRecord {
    /// Record ID
    pub id: u32,
    /// Category
    pub category: u8,
    /// Attributes
    pub attributes: CmpAttributes,
    /// Protocol version
    pub protocol_version: u8,
    /// Message type
    pub message_type: CmpMessageType,
    /// Sender ID
    pub sender_id: String,
    /// Recipient ID
    pub recipient_id: String,
    /// Subject
    pub subject: String,
    /// Body
    pub body: String,
    /// Timestamp
    pub timestamp: PalmDateTime,
    /// Priority
    pub priority: CmpPriority,
    /// Status
    pub status: CmpStatus,
    /// Custom fields
    pub custom: Vec<CmpField>,
}

/// CMP attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmpAttributes(u8);

impl CmpAttributes {
    pub const SECRET: u8 = 0x02;
    pub const BUSY: u8 = 0x20;
    pub const ARCHIVE: u8 = 0x10;
    pub const ENCRYPTED: u8 = 0x40;

    pub fn is_secret(&self) -> bool {
        (self.0 & Self::SECRET) != 0
    }
    pub fn is_busy(&self) -> bool {
        (self.0 & Self::BUSY) != 0
    }
    pub fn is_archived(&self) -> bool {
        (self.0 & Self::ARCHIVE) != 0
    }
    pub fn is_encrypted(&self) -> bool {
        (self.0 & Self::ENCRYPTED) != 0
    }
}

/// CMP message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CmpMessageType {
    Text = 0,
    Data = 1,
    Command = 2,
    Response = 3,
    Ack = 4,
    Nack = 5,
    KeepAlive = 6,
    Handshake = 7,
}

impl CmpMessageType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => CmpMessageType::Text,
            1 => CmpMessageType::Data,
            2 => CmpMessageType::Command,
            3 => CmpMessageType::Response,
            4 => CmpMessageType::Ack,
            5 => CmpMessageType::Nack,
            6 => CmpMessageType::KeepAlive,
            7 => CmpMessageType::Handshake,
            _ => CmpMessageType::Text,
        }
    }
}

/// CMP priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CmpPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl CmpPriority {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => CmpPriority::Low,
            2 => CmpPriority::High,
            3 => CmpPriority::Urgent,
            _ => CmpPriority::Normal,
        }
    }
}

/// CMP status values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CmpStatus {
    Pending = 0,
    Sent = 1,
    Delivered = 2,
    Read = 3,
    Failed = 4,
    Deleted = 5,
}

impl CmpStatus {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => CmpStatus::Pending,
            1 => CmpStatus::Sent,
            2 => CmpStatus::Delivered,
            3 => CmpStatus::Read,
            4 => CmpStatus::Failed,
            _ => CmpStatus::Deleted,
        }
    }
}

/// Custom field
#[derive(Debug, Clone)]
pub struct CmpField {
    /// Field name
    pub name: String,
    /// Field value
    pub value: String,
}

/// CMP packet header
#[derive(Debug, Clone)]
pub struct CmpHeader {
    /// Protocol version
    pub version: u8,
    /// Message type
    pub message_type: CmpMessageType,
    /// Sequence number
    pub sequence: u16,
    /// Flags
    pub flags: u8,
    /// Payload length
    pub length: u16,
}

impl CmpHeader {
    /// Pack header to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.version);
        bytes.push(self.message_type as u8);
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes.push(self.flags);
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes
    }

    /// Parse header from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 7 {
            return Err(PilotError::InvalidData("CMP header too short".into()));
        }

        Ok(Self {
            version: data[0],
            message_type: CmpMessageType::from_u8(data[1]),
            sequence: u16::from_be_bytes([data[2], data[3]]),
            flags: data[4],
            length: u16::from_be_bytes([data[5], data[6]]),
        })
    }
}

/// CMP session
#[derive(Debug, Clone)]
pub struct CmpSession {
    /// Session ID
    pub id: String,
    /// State
    pub state: CmpSessionState,
    /// Last activity
    pub last_activity: PalmDateTime,
    /// Sequence number
    pub sequence: u16,
    /// Messages sent
    pub messages_sent: u32,
    /// Messages received
    pub messages_received: u32,
}

/// CMP session states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpSessionState {
    Disconnected,
    Connecting,
    Handshake,
    Connected,
    Active,
    Closing,
    Error,
}

impl Default for CmpRecord {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            attributes: CmpAttributes(0),
            protocol_version: 1,
            message_type: CmpMessageType::Text,
            sender_id: String::new(),
            recipient_id: String::new(),
            subject: String::new(),
            body: String::new(),
            timestamp: PalmDateTime::now(),
            priority: CmpPriority::Normal,
            status: CmpStatus::Pending,
            custom: Vec::new(),
        }
    }
}

impl CmpRecord {
    /// Parse from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 15 {
            return Err(PilotError::InvalidData("CMP record too short".into()));
        }

        let mut record = CmpRecord::default();
        let mut offset = 0;

        // Protocol version
        record.protocol_version = data[offset];
        offset += 1;

        // Message type
        record.message_type = CmpMessageType::from_u8(data[offset]);
        offset += 1;

        // Priority
        record.priority = CmpPriority::from_u8(data[offset]);
        offset += 1;

        // Status
        record.status = CmpStatus::from_u8(data[offset]);
        offset += 1;

        // Timestamp
        let timestamp_val = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;
        record.timestamp = PalmDateTime::from_palm(timestamp_val);

        // Parse strings
        let (sender, new_offset) = Self::parse_string(data, offset)?;
        record.sender_id = sender;
        offset = new_offset;

        let (recipient, new_offset) = Self::parse_string(data, offset)?;
        record.recipient_id = recipient;
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

        // Protocol version
        data.push(self.protocol_version);

        // Message type
        data.push(self.message_type as u8);

        // Priority
        data.push(self.priority as u8);

        // Status
        data.push(self.status as u8);

        // Timestamp
        data.extend_from_slice(&self.timestamp.to_palm().to_be_bytes());

        // Strings
        data.extend_from_slice(&Self::pack_string(&self.sender_id));
        data.extend_from_slice(&Self::pack_string(&self.recipient_id));
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

    /// Check if message is encrypted
    pub fn is_encrypted(&self) -> bool {
        self.attributes.is_encrypted()
    }

    /// Get total size
    pub fn total_size(&self) -> usize {
        self.pack().len()
    }
}

/// CMP constants
pub mod constants {
    /// Current protocol version
    pub const CMP_VERSION: u8 = 1;

    /// Maximum message size
    pub const MAX_MESSAGE_SIZE: usize = 65535;

    /// Default timeout (ms)
    pub const DEFAULT_TIMEOUT: u32 = 30000;

    /// Keep-alive interval (ms)
    pub const KEEPALIVE_INTERVAL: u32 = 60000;

    /// Maximum retries
    pub const MAX_RETRIES: u8 = 3;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_message_type() {
        assert_eq!(CmpMessageType::from_u8(0), CmpMessageType::Text);
        assert_eq!(CmpMessageType::from_u8(4), CmpMessageType::Ack);
        assert_eq!(CmpMessageType::from_u8(10), CmpMessageType::Text);
    }

    #[test]
    fn test_cmp_priority() {
        assert_eq!(CmpPriority::from_u8(0), CmpPriority::Low);
        assert_eq!(CmpPriority::from_u8(1), CmpPriority::Normal);
        assert_eq!(CmpPriority::from_u8(3), CmpPriority::Urgent);
    }

    #[test]
    fn test_cmp_status() {
        assert_eq!(CmpStatus::from_u8(0), CmpStatus::Pending);
        assert_eq!(CmpStatus::from_u8(3), CmpStatus::Read);
        assert_eq!(CmpStatus::from_u8(10), CmpStatus::Deleted);
    }

    #[test]
    fn test_cmp_attributes() {
        let attrs = CmpAttributes(CmpAttributes::SECRET | CmpAttributes::ENCRYPTED);
        assert!(attrs.is_secret());
        assert!(attrs.is_encrypted());
        assert!(!attrs.is_archived());
    }

    #[test]
    fn test_cmp_header_pack_parse() {
        let header = CmpHeader {
            version: 1,
            message_type: CmpMessageType::Text,
            sequence: 1234,
            flags: 0,
            length: 100,
        };

        let bytes = header.pack();
        let parsed = CmpHeader::parse(&bytes).unwrap();

        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.message_type, CmpMessageType::Text);
        assert_eq!(parsed.sequence, 1234);
        assert_eq!(parsed.length, 100);
    }

    #[test]
    fn test_cmp_record_pack_parse() {
        let mut record = CmpRecord::default();
        record.sender_id = "alice".to_string();
        record.recipient_id = "bob".to_string();
        record.subject = "Test message".to_string();
        record.body = "Hello, World!".to_string();
        record.message_type = CmpMessageType::Text;

        let packed = record.pack();
        let parsed = CmpRecord::parse(&packed).unwrap();

        assert_eq!(parsed.sender_id, "alice");
        assert_eq!(parsed.recipient_id, "bob");
        assert_eq!(parsed.subject, "Test message");
        assert_eq!(parsed.body, "Hello, World!");
    }

    #[test]
    fn test_is_encrypted() {
        let mut record = CmpRecord::default();
        assert!(!record.is_encrypted());

        record.attributes = CmpAttributes(CmpAttributes::ENCRYPTED);
        assert!(record.is_encrypted());
    }
}
