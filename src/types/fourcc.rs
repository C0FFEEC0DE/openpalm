//! Four-character code (FourCC) type

use std::fmt;

/// A four-character code used in Palm OS for types and creators
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Default)]
pub struct FourCharCode(pub u32);

impl FourCharCode {
    /// Create a FourCC from raw bytes
    /// The bytes are stored in big-endian format (first byte is most significant)
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        FourCharCode::from_u32(
            ((bytes[0] as u32) << 24) |
            ((bytes[1] as u32) << 16) |
            ((bytes[2] as u32) << 8) |
            (bytes[3] as u32)
        )
    }

    /// Create a FourCC from a u32 value
    pub fn from_u32(value: u32) -> Self {
        FourCharCode(value)
    }

    /// Get the raw u32 value
    pub fn to_u32(&self) -> u32 {
        self.0
    }

    /// Get the bytes as a slice (big-endian order)
    pub fn as_bytes(&self) -> [u8; 4] {
        [
            ((self.0 >> 24) & 0xFF) as u8,
            ((self.0 >> 16) & 0xFF) as u8,
            ((self.0 >> 8) & 0xFF) as u8,
            (self.0 & 0xFF) as u8,
        ]
    }

    /// Get the bytes as a string
    pub fn as_str(&self) -> String {
        String::from_utf8_lossy(&self.as_bytes()).into_owned()
    }

    /// Create from a string slice (must be exactly 4 bytes)
    pub fn from_str(s: &str) -> Self {
        assert_eq!(s.len(), 4, "FourCC must be exactly 4 bytes");
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(s.as_bytes());
        FourCharCode::from_bytes(bytes)
    }
}


impl fmt::Debug for FourCharCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FourCharCode({:?})", self.as_str())
    }
}

impl fmt::Display for FourCharCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", self.as_str())
    }
}

impl From<[u8; 4]> for FourCharCode {
    fn from(bytes: [u8; 4]) -> Self {
        FourCharCode::from_bytes(bytes)
    }
}

impl From<u32> for FourCharCode {
    fn from(value: u32) -> Self {
        FourCharCode::from_u32(value)
    }
}

impl From<FourCharCode> for u32 {
    fn from(code: FourCharCode) -> Self {
        code.to_u32()
    }
}

// Common database types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    Application,
    Data,
    System,
    Resource,
    Unknown(FourCharCode),
}

impl DatabaseType {
    /// Get the FourCC for this database type
    pub fn fourcc(&self) -> FourCharCode {
        match self {
            DatabaseType::Application => FourCharCode::from_bytes(*b"appl"),
            DatabaseType::Data => FourCharCode::from_bytes(*b"DATA"),
            DatabaseType::System => FourCharCode::from_bytes(*b"syst"),
            DatabaseType::Resource => FourCharCode::from_bytes(*b"rsrc"),
            DatabaseType::Unknown(code) => *code,
        }
    }

    /// Create from a FourCC
    pub fn from_fourcc(code: FourCharCode) -> Self {
        match code.to_u32() {
            0x6170706C => DatabaseType::Application,
            0x44415441 => DatabaseType::Data,
            0x73797374 => DatabaseType::System,
            0x72737263 => DatabaseType::Resource,
            _ => DatabaseType::Unknown(code),
        }
    }
}

// Common database creators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseCreator {
    AddressBook,
    Datebook,
    Todo,
    Memo,
    Mail,
    Unknown(FourCharCode),
}

impl DatabaseCreator {
    /// Get the FourCC for this creator
    pub fn fourcc(&self) -> FourCharCode {
        match self {
            DatabaseCreator::AddressBook => FourCharCode::from_bytes(*b"ADDR"),
            DatabaseCreator::Datebook => FourCharCode::from_bytes(*b"date"),
            DatabaseCreator::Todo => FourCharCode::from_bytes(*b"todo"),
            DatabaseCreator::Memo => FourCharCode::from_bytes(*b"memo"),
            DatabaseCreator::Mail => FourCharCode::from_bytes(*b"mlmp"),
            DatabaseCreator::Unknown(code) => *code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        let code = FourCharCode::from_bytes(*b"appl");
        assert_eq!(code.to_u32(), 0x6170706C);
    }

    #[test]
    fn test_to_bytes() {
        let code = FourCharCode::from_u32(0x6170706C);
        assert_eq!(code.as_bytes(), *b"appl");
    }

    #[test]
    fn test_as_str() {
        let code = FourCharCode::from_bytes(*b"DATA");
        assert_eq!(code.as_str(), "DATA");
    }

    #[test]
    fn test_roundtrip() {
        let original = FourCharCode::from_bytes(*b"abcd");
        let bytes = original.as_bytes();
        let restored = FourCharCode::from_bytes(bytes);
        assert_eq!(original, restored);
    }
}
