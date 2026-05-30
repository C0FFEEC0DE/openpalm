//! String utilities for Palm OS record parsing
//!
//! This module provides common string parsing utilities used across
//! all record modules for parsing null-terminated strings.

use crate::error::{PilotError, Result};

/// Decode Palm OS string bytes using CP1252 (Windows Western encoding).
/// Palm OS devices typically store strings in CP1252, not UTF-8.
/// Falls back to lossy UTF-8 for bytes that are not valid CP1252.
pub fn decode_palm_string(bytes: &[u8]) -> String {
    // CP1252 is a superset of ISO-8859-1 with additional characters in 0x80-0x9F
    let (cow, _encoding_used, had_errors) = encoding_rs::WINDOWS_1252.decode(bytes);
    if had_errors {
        // Should not happen for CP1252 (it's a single-byte encoding for all 256 values)
        String::from_utf8_lossy(bytes).to_string()
    } else {
        cow.to_string()
    }
}

/// Parse a null-terminated string from byte data
///
/// # Arguments
/// * `data` - The byte buffer to parse from
/// * `offset` - Starting offset in the buffer
///
/// # Returns
/// * `Ok((String, usize))` - The parsed string and new offset after the null terminator
pub fn parse_pstring(data: &[u8], offset: usize) -> Result<(String, usize)> {
    if offset >= data.len() {
        return Err(PilotError::InvalidData("Offset beyond data length".into()));
    }
    
    let mut end = offset;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    
    let s = String::from_utf8_lossy(&data[offset..end]).to_string();
    let new_offset = if end < data.len() { end + 1 } else { end };
    
    Ok((s, new_offset))
}

/// Parse a length-prefixed string (Pascal-style)
///
/// # Arguments
/// * `data` - The byte buffer to parse from
/// * `offset` - Starting offset in the buffer
///
/// # Returns
/// * `Ok((String, usize))` - The parsed string and new offset
pub fn parse_lpstring(data: &[u8], offset: usize) -> Result<(String, usize)> {
    if offset >= data.len() {
        return Err(PilotError::InvalidData("Offset beyond data length".into()));
    }
    
    let len = data[offset] as usize;
    
    if offset + 1 + len > data.len() {
        return Err(PilotError::InvalidData("Pascal string exceeds buffer".into()));
    }
    
    let s = String::from_utf8_lossy(&data[offset + 1..offset + 1 + len]).to_string();
    Ok((s, offset + 1 + len))
}

/// Pack a string as null-terminated (C-style)
///
/// # Arguments
/// * `s` - The string to pack
///
/// # Returns
/// * `Vec<u8>` - The packed bytes with null terminator
pub fn pack_pstring(s: &str) -> Vec<u8> {
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

/// Pack a string as length-prefixed (Pascal-style)
///
/// # Arguments
/// * `s` - The string to pack
///
/// # Returns
/// * `Vec<u8>` - The packed bytes with length prefix
pub fn pack_lpstring(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let len = bytes.len().min(255) as u8;
    
    let mut result = Vec::with_capacity(1 + bytes.len());
    result.push(len);
    result.extend_from_slice(bytes);
    result
}

/// Parse multiple null-terminated strings until end of data or max count
///
/// # Arguments
/// * `data` - The byte buffer to parse from
/// * `offset` - Starting offset in the buffer
/// * `max_count` - Maximum number of strings to parse
///
/// # Returns
/// * `Ok(Vec<String>)` - The parsed strings
pub fn parse_string_list(data: &[u8], offset: usize, max_count: usize) -> Result<(Vec<String>, usize)> {
    let mut strings = Vec::new();
    let mut current_offset = offset;

    for _ in 0..max_count {
        if current_offset >= data.len() {
            break;
        }

        // Check for end marker (empty string at end) or double null
        if data[current_offset] == 0 {
            current_offset += 1;
            break;
        }

        let (s, new_offset) = parse_pstring(data, current_offset)?;
        if s.is_empty() {
            break;
        }
        strings.push(s);
        current_offset = new_offset;
    }

    Ok((strings, current_offset))
}

/// Pack multiple strings as null-terminated list (double-null terminated)
///
/// # Arguments
/// * `strings` - The strings to pack
///
/// # Returns
/// * `Vec<u8>` - The packed strings with double null terminator
pub fn pack_string_list(strings: &[String]) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    for s in strings {
        bytes.extend(pack_pstring(s));
    }
    
    // Double null terminator to mark end of list
    bytes.push(0);
    bytes.push(0);
    
    bytes
}

/// Calculate the packed size of a null-terminated string
pub fn pstring_size(s: &str) -> usize {
    s.len() + 1 // string bytes + null terminator
}

/// Calculate the packed size of a list of null-terminated strings
pub fn string_list_size(strings: &[String]) -> usize {
    strings.iter().map(|s| pstring_size(s)).sum::<usize>() + 2 // +2 for double null
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pstring() {
        let data = b"Hello\0World\0";
        let (s, offset) = parse_pstring(data, 0).unwrap();
        assert_eq!(s, "Hello");
        assert_eq!(offset, 6);
        
        let (s2, offset2) = parse_pstring(data, offset).unwrap();
        assert_eq!(s2, "World");
        assert_eq!(offset2, 12);
    }

    #[test]
    fn test_pack_pstring() {
        let packed = pack_pstring("Test");
        assert_eq!(packed, b"Test\0");  // "Test" + null terminator
        assert_eq!(packed.len(), 5);
    }

    #[test]
    fn test_parse_lpstring() {
        let data = b"\x05Hello";
        let (s, offset) = parse_lpstring(data, 0).unwrap();
        assert_eq!(s, "Hello");
        assert_eq!(offset, 6);
    }

    #[test]
    fn test_parse_string_list() {
        let data = b"Alice\0Bob\0Charlie\0";
        let (strings, offset) = parse_string_list(data, 0, 10).unwrap();
        assert_eq!(strings.len(), 3);
        assert_eq!(strings[0], "Alice");
        assert_eq!(strings[1], "Bob");
        assert_eq!(strings[2], "Charlie");
        assert_eq!(offset, 18);
    }

    #[test]
    fn test_pack_string_list() {
        let strings = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let packed = pack_string_list(&strings);
        // A + null + B + null + C + null + null (double null terminator)
        // = [65, 0, 66, 0, 67, 0, 0, 0] = 8 bytes
        assert_eq!(packed, b"A\0B\0C\0\0\0");
        assert_eq!(packed.len(), 8);
    }

    #[test]
    fn test_empty_string() {
        let data = b"\0";
        let (s, _) = parse_pstring(data, 0).unwrap();
        assert_eq!(s, "");
    }

    #[test]
    fn test_unicode_string() {
        let data = b"Hello\xC3\xA4\xC3\xB6\xC3\xBC\0"; // German umlauts
        let (s, _) = parse_pstring(data, 0).unwrap();
        assert_eq!(s, "Helloäöü");
    }

    #[test]
    fn test_decode_palm_cp1252() {
        // Euro sign € in CP1252 = 0x80
        let bytes = b"\x80";
        let s = decode_palm_string(bytes);
        assert_eq!(s, "€");

        // Smart quotes in CP1252
        let bytes2 = b"\x91Hello\x92"; // ‘Hello’ (smart quotes)
        let s2 = decode_palm_string(bytes2);
        assert_eq!(s2, "\u{2018}Hello\u{2019}"); // U+2018/U+2019 left/right single quotation marks
    }

    #[test]
    fn test_pstring_size() {
        assert_eq!(pstring_size("Test"), 5);
        assert_eq!(pstring_size(""), 1);
    }

    #[test]
    fn test_string_list_size() {
        let strings = vec!["A".to_string(), "BB".to_string()];
        // A + null + BB + null + double null = 1+1+2+1+2 = 7
        assert_eq!(string_list_size(&strings), 7);
    }
}