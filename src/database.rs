//! Database layer for Palm OS databases
//!
//! This module provides database operations and types for Palm OS databases,
//! including database headers, records, and app info blocks.

use crate::error::{PilotError, Result};
use crate::types::{FourCharCode, DatabaseFlags, RecordFlags, PalmDateTime, OpenMode};
use crate::types::buffer::PiBuffer;

/// Database handle (returned by open operations)
pub type DatabaseHandle = u8;

/// Record ID (unique within database)
pub type RecordId = u32;

/// Card number
pub type CardNo = u16;

/// Maximum database name length
pub const MAX_DBP_NAME_LEN: usize = 32;

/// Database info structure
#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    /// Flags (from database header)
    pub flags: DatabaseFlags,
    /// Database type (creator code)
    pub db_type: FourCharCode,
    /// Database creator
    pub creator: FourCharCode,
    /// Card number
    pub card_no: CardNo,
    /// Database ID
    pub db_id: u32,
    /// Creation time
    pub created: PalmDateTime,
    /// Modification time
    pub modified: PalmDateTime,
    /// Last backup time
    pub backup_date: PalmDateTime,
    /// Modification number
    pub mod_num: u32,
    /// App info dirty flag
    pub app_info_dirty: bool,
    /// Sort info dirty flag
    pub sort_info_dirty: bool,
    /// Total bytes used (including header)
    pub total_bytes: u32,
    /// Data bytes (excluding header and records)
    pub data_bytes: u32,
    /// Number of records
    pub num_records: u32,
    /// Unique ID seed
    pub unique_id_seed: u32,
    /// Database name
    pub name: String,
}

impl Default for DatabaseInfo {
    fn default() -> Self {
        Self {
            flags: DatabaseFlags::empty(),
            db_type: FourCharCode::default(),
            creator: FourCharCode::default(),
            card_no: 0,
            db_id: 0,
            created: PalmDateTime::default(),
            modified: PalmDateTime::default(),
            backup_date: PalmDateTime::default(),
            mod_num: 0,
            app_info_dirty: false,
            sort_info_dirty: false,
            total_bytes: 0,
            data_bytes: 0,
            num_records: 0,
            unique_id_seed: 0,
            name: String::new(),
        }
    }
}

/// A database record
#[derive(Debug, Clone)]
pub struct Record {
    /// Record ID (unique within database)
    pub id: RecordId,
    /// Index within database
    pub index: u32,
    /// Record attributes
    pub attributes: RecordFlags,
    /// Record data
    pub data: Vec<u8>,
    /// Sort key (optional)
    pub sort_key: Option<Vec<u8>>,
}

impl Record {
    /// Create a new record
    pub fn new(id: RecordId, data: Vec<u8>) -> Self {
        Self {
            id,
            index: 0,
            attributes: RecordFlags::empty(),
            data,
            sort_key: None,
        }
    }
    
    /// Create with attributes
    pub fn with_attributes(mut self, attrs: RecordFlags) -> Self {
        self.attributes = attrs;
        self
    }
    
    /// Get the record data as UTF-8 string (if valid UTF-8)
    pub fn data_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }
    
    /// Check if record is deleted
    pub fn is_deleted(&self) -> bool {
        self.attributes.contains(RecordFlags::DELETED)
    }
    
    /// Check if record is dirty
    pub fn is_dirty(&self) -> bool {
        self.attributes.contains(RecordFlags::DIRTY)
    }
    
    /// Check if record is busy
    pub fn is_busy(&self) -> bool {
        self.attributes.contains(RecordFlags::BUSY)
    }
}

/// Database header (on-device format)
#[derive(Debug, Clone)]
pub struct DatabaseHeader {
    /// Named database header (next 78 bytes)
    /// "Name" - database name (32 bytes, null-terminated)
    pub name: [u8; 32],
    /// Flags
    pub flags: u16,
    /// Version
    pub version: u16,
    /// Creation time
    pub created: u32,
    /// Modification time
    pub modified: u32,
    /// Backup time
    pub backup: u32,
    /// Modification number
    pub mod_num: u32,
    /// App info ID
    pub app_info_id: u32,
    /// Sort info ID
    pub sort_info_id: u32,
    /// Database type
    pub db_type: u32,
    /// Creator ID
    pub creator: u32,
    /// Unique ID seed
    pub unique_id_seed: u32,
    /// Next record list ID
    pub next_rec_list_id: u32,
    /// Number of records
    pub num_records: u16,
    /// Unique record ID seed (for this header)
    pub unique_record_seed: u16,
    /// Reserved (2 bytes)
    _reserved: [u8; 2],
}

impl Default for DatabaseHeader {
    fn default() -> Self {
        Self {
            name: [0u8; 32],
            flags: 0,
            version: 0,
            created: 0,
            modified: 0,
            backup: 0,
            mod_num: 0,
            app_info_id: 0,
            sort_info_id: 0,
            db_type: 0,
            creator: 0,
            unique_id_seed: 0,
            next_rec_list_id: 0,
            num_records: 0,
            unique_record_seed: 0,
            _reserved: [0u8; 2],
        }
    }
}

impl DatabaseHeader {
    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 86 {
            return Err(PilotError::DlpBufSize);
        }
        
        let mut header = Self::default();
        header.name.copy_from_slice(&data[..32]);
        header.flags = u16::from_be_bytes([data[32], data[33]]);
        header.version = u16::from_be_bytes([data[34], data[35]]);
        header.created = u32::from_be_bytes([data[36], data[37], data[38], data[39]]);
        header.modified = u32::from_be_bytes([data[40], data[41], data[42], data[43]]);
        header.backup = u32::from_be_bytes([data[44], data[45], data[46], data[47]]);
        header.mod_num = u32::from_be_bytes([data[48], data[49], data[50], data[51]]);
        header.app_info_id = u32::from_be_bytes([data[52], data[53], data[54], data[55]]);
        header.sort_info_id = u32::from_be_bytes([data[56], data[57], data[58], data[59]]);
        header.db_type = u32::from_be_bytes([data[60], data[61], data[62], data[63]]);
        header.creator = u32::from_be_bytes([data[64], data[65], data[66], data[67]]);
        header.unique_id_seed = u32::from_be_bytes([data[68], data[69], data[70], data[71]]);
        header.next_rec_list_id = u32::from_be_bytes([data[72], data[73], data[74], data[75]]);
        header.num_records = u16::from_be_bytes([data[76], data[77]]);
        header.unique_record_seed = u16::from_be_bytes([data[78], data[79]]);
        header._reserved.copy_from_slice(&data[80..82]);
        
        // Skip padding bytes 82..86
        
        Ok(header)
    }
    
    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = vec![0u8; 86];
        
        data[..32].copy_from_slice(&self.name);
        data[32..34].copy_from_slice(&self.flags.to_be_bytes());
        data[34..36].copy_from_slice(&self.version.to_be_bytes());
        data[36..40].copy_from_slice(&self.created.to_be_bytes());
        data[40..44].copy_from_slice(&self.modified.to_be_bytes());
        data[44..48].copy_from_slice(&self.backup.to_be_bytes());
        data[48..52].copy_from_slice(&self.mod_num.to_be_bytes());
        data[52..56].copy_from_slice(&self.app_info_id.to_be_bytes());
        data[56..60].copy_from_slice(&self.sort_info_id.to_be_bytes());
        data[60..64].copy_from_slice(&self.db_type.to_be_bytes());
        data[64..68].copy_from_slice(&self.creator.to_be_bytes());
        data[68..72].copy_from_slice(&self.unique_id_seed.to_be_bytes());
        data[72..76].copy_from_slice(&self.next_rec_list_id.to_be_bytes());
        data[76..78].copy_from_slice(&self.num_records.to_be_bytes());
        data[78..80].copy_from_slice(&self.unique_record_seed.to_be_bytes());
        data[80..82].copy_from_slice(&self._reserved);
        // Pad to 86 bytes total
        data[82..86].copy_from_slice(&[0u8; 4]);
        
        data
    }
    
    /// Get database name as string
    pub fn name_str(&self) -> String {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(32);
        String::from_utf8_lossy(&self.name[..end]).to_string()
    }
}

/// Record entry in database (index table)
#[derive(Debug, Clone)]
pub struct RecordEntry {
    /// Local chunk ID for data
    pub local_chunk_id: u32,
    /// Record attributes
    pub attributes: u8,
    /// Unique ID (3 bytes)
    pub unique_id: [u8; 3],
}

impl RecordEntry {
    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(PilotError::DlpBufSize);
        }
        
        Ok(Self {
            local_chunk_id: u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
            attributes: data[4],
            unique_id: [data[5], data[6], data[7]],
        })
    }
    
    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = vec![0u8; 8];
        
        data[..4].copy_from_slice(&self.local_chunk_id.to_be_bytes());
        data[4] = self.attributes;
        data[5..8].copy_from_slice(&self.unique_id);
        
        data
    }
    
    /// Get the full record ID from unique_id
    pub fn record_id(&self) -> RecordId {
        // Record ID is based on unique ID allocation algorithm
        u32::from_le_bytes([self.unique_id[0], self.unique_id[1], self.unique_id[2], 0])
    }
}

/// Application info block
#[derive(Debug, Clone, Default)]
pub struct AppInfo {
    /// Category assignment version
    pub version: u16,
    /// Reserved
    pub reserved: u16,
    /// Categories (16 entries, each 16 bytes + name)
    pub categories: Vec<Category>,
    /// Last unique ID used
    pub last_unique_id: u16,
    /// App-specific data
    pub data: Vec<u8>,
}

impl AppInfo {
    /// Create a new app info
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Parse from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(PilotError::DlpBufSize);
        }
        
        let mut info = Self::default();
        info.version = u16::from_be_bytes([data[0], data[1]]);
        info.reserved = u16::from_be_bytes([data[2], data[3]]);
        info.data = data[4..].to_vec();
        
        Ok(info)
    }
    
    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(4 + self.data.len());
        data.extend_from_slice(&self.version.to_be_bytes());
        data.extend_from_slice(&self.reserved.to_be_bytes());
        data.extend_from_slice(&self.data);
        data
    }
}

/// Category definition
#[derive(Debug, Clone)]
pub struct Category {
    /// Category ID
    pub id: u8,
    /// Category name (16 bytes, null-terminated)
    pub name: [u8; 16],
    /// Category flags
    pub flags: u8,
    /// Reserved
    pub reserved: u8,
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: 0,
            name: [0u8; 16],
            flags: 0,
            reserved: 0,
        }
    }
}

/// Database wrapper
#[derive(Debug)]
pub struct Database {
    /// Handle (from open operation)
    pub handle: DatabaseHandle,
    /// Card number
    pub card_no: CardNo,
    /// Database info
    pub info: DatabaseInfo,
    /// Local ID of database
    pub local_id: u32,
    /// Open mode
    pub mode: OpenMode,
}

impl Database {
    /// Create a new database wrapper
    pub fn new(handle: DatabaseHandle, card_no: CardNo, info: DatabaseInfo) -> Self {
        Self {
            handle,
            card_no,
            info,
            local_id: 0,
            mode: OpenMode::READ,
        }
    }
    
    /// Get database name
    pub fn name(&self) -> &str {
        &self.info.name
    }
    
    /// Get number of records
    pub fn record_count(&self) -> u32 {
        self.info.num_records
    }
    
    /// Check if database is open
    pub fn is_open(&self) -> bool {
        self.handle != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_basic() {
        let record = Record::new(0x10000000, vec![0x01, 0x02, 0x03]);
        
        assert_eq!(record.id, 0x10000000);
        assert_eq!(record.data, vec![0x01, 0x02, 0x03]);
        assert!(!record.is_deleted());
    }

    #[test]
    fn test_database_header() {
        let mut header = DatabaseHeader::default();
        // Set name as null-terminated string "TestDB" (5 chars + null = 6 bytes)
        header.name[..6].copy_from_slice(&b"TestDB\0"[..6]);
        header.num_records = 10;
        
        let bytes = header.to_bytes();
        let parsed = DatabaseHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed.name_str(), "TestDB");
        assert_eq!(parsed.num_records, 10);
    }

    #[test]
    fn test_record_entry() {
        let entry = RecordEntry {
            local_chunk_id: 0x11223344,
            attributes: 0,
            unique_id: [0x10, 0x20, 0x30],
        };
        
        let bytes = entry.to_bytes();
        let parsed = RecordEntry::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed.local_chunk_id, 0x11223344);
        assert_eq!(parsed.unique_id, [0x10, 0x20, 0x30]);
    }
}
