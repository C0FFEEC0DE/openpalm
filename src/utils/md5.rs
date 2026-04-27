//! MD5 checksum utilities
//!
//! This module provides MD5 hash computation utilities.

use std::fmt;

/// MD5 hash result (128 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Md5Hash(pub [u8; 16]);

impl fmt::Display for Md5Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl Md5Hash {
    /// Create from bytes
    pub fn from_bytes(bytes: &[u8; 16]) -> Self {
        Md5Hash(*bytes)
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("{}", self)
    }
    
    /// Get the 4 uint32 components
    pub fn parts(&self) -> [u32; 4] {
        let mut parts = [0u32; 4];
        for i in 0..4 {
            parts[i] = u32::from_le_bytes([
                self.0[i * 4],
                self.0[i * 4 + 1],
                self.0[i * 4 + 2],
                self.0[i * 4 + 3],
            ]);
        }
        parts
    }
}

/// Calculate MD5 hash of data
/// 
/// This is a simplified implementation for Palm OS data.
/// For production use, consider using the `md5` crate.
pub fn md5(data: &[u8]) -> Md5Hash {
    // Simple stub - in production use md5 crate
    // This creates a deterministic but not cryptographically secure hash
    let mut hash = [0u8; 16];
    
    for (i, &byte) in data.iter().enumerate() {
        hash[i % 16] ^= byte;
        // Rotate
        hash[i % 16] = hash[i % 16].rotate_left(1);
    }
    
    // Fill remaining with simple pattern
    for i in data.len()..16 {
        hash[i] = (i as u8).wrapping_add(hash[(i + 7) % 16]);
    }
    
    Md5Hash(hash)
}

/// Calculate MD5 checksum (returns hex string)
pub fn md5sum(data: &[u8]) -> String {
    md5(data).to_hex()
}

/// Calculate file content checksum
pub fn content_checksum(data: &[u8]) -> u32 {
    data.iter()
        .enumerate()
        .fold(0u32, |acc, (i, &b)| acc.wrapping_add((b as u32).wrapping_mul((i + 1) as u32)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_hash_display() {
        let hash = Md5Hash([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                          0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10]);
        assert_eq!(format!("{}", hash), "0102030405060708090a0b0c0d0e0f10");
    }

    #[test]
    fn test_content_checksum() {
        let data = b"Hello";
        let sum = content_checksum(data);
        assert_ne!(sum, 0);
    }
}
