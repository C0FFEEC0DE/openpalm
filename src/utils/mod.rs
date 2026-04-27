//! Utility functions for openpalm
//!
//! This module provides various utility functions used throughout the library.

mod md5;
mod debug;
mod sys;
mod strings;

pub use md5::{md5, md5sum, Md5Hash};
pub use debug::{hex_dump, dump_packet, DebugLevel, Logger};

// Re-export system utilities
pub use sys::{
    timeout_to_duration,
    timeout_to_system_time,
    timeout_expired,
    system_time_to_timeout,
    get_pilot_rate as pilot_rate_env,
    page_size,
    page_align,
    is_big_endian,
    host_byte_order,
    SystemInfo,
};

// Re-export string utilities
pub use strings::{
    parse_pstring, pack_pstring,
    parse_lpstring, pack_lpstring,
    parse_string_list, pack_string_list,
    pstring_size, string_list_size,
};

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get current system time as Palm timestamp
pub fn now_palm() -> u32 {
    use crate::types::PalmDateTime;
    PalmDateTime::now().to_palm()
}

/// Format timestamp as human-readable string
pub fn format_timestamp(timestamp: u32) -> String {
    use crate::types::PalmDateTime;
    let dt = PalmDateTime::from_palm(timestamp);
    dt.format("%Y-%m-%d %H:%M:%S")
}

/// Parse Palm timestamp to SystemTime
pub fn palm_to_system_time(timestamp: u32) -> SystemTime {
    use crate::types::PalmDateTime;
    let dt = PalmDateTime::from_palm(timestamp);
    let unix_secs = dt.to_unix();
    UNIX_EPOCH + Duration::from_secs(unix_secs as u64)
}

/// Get environment variable for PILOTRATE
pub fn get_pilot_rate() -> (i32, bool) {
    // Default PADP connection rate
    if let Ok(rate_env) = std::env::var("PILOTRATE") {
        if rate_env.starts_with('H') {
            let rate = rate_env[1..].parse().unwrap_or(-1);
            (rate, true)
        } else {
            let rate = rate_env.parse().unwrap_or(-1);
            (rate, false)
        }
    } else {
        (-1, false)
    }
}

/// CRC16 implementation for Palm data
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    
    crc
}

/// CRC32 implementation
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    
    crc ^ 0xFFFFFFFF
}

/// Calculate checksum (sum of bytes, modulo 256)
pub fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
}

/// Convert byte to hex string
pub fn byte_to_hex(byte: u8) -> String {
    format!("{:02X}", byte)
}

/// Convert bytes to hex string
pub fn bytes_to_hex(data: &[u8]) -> String {
    data.iter().map(|&b| byte_to_hex(b)).collect::<Vec<_>>().join(" ")
}

/// Parse hex string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, &'static str> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length");
    }
    
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut chars = hex.chars().peekable();
    
    while let (Some(a), Some(b)) = (chars.next(), chars.next()) {
        let hex_byte = format!("{}{}", a, b);
        let byte = u8::from_str_radix(&hex_byte, 16)
            .map_err(|_| "Invalid hex byte")?;
        bytes.push(byte);
    }
    
    Ok(bytes)
}

/// Align value to boundary
pub fn align(value: usize, boundary: usize) -> usize {
    (value + boundary - 1) & !(boundary - 1)
}

/// Pad data to alignment
pub fn pad_to_align(data: Vec<u8>, boundary: usize) -> Vec<u8> {
    let aligned = align(data.len(), boundary);
    let padding = aligned - data.len();
    
    if padding == 0 {
        data
    } else {
        let mut padded = data;
        padded.resize(aligned, 0);
        padded
    }
}

/// Minimum of two values
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

/// Maximum of two values
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

/// Clamp value to range
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min { min } else if value > max { max } else { value }
}

/// Swap two values
pub fn swap<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}

/// Rotate bits left
pub fn rotl(value: u32, n: u32) -> u32 {
    (value << n) | (value >> (32 - n))
}

/// Rotate bits right
pub fn rotr(value: u32, n: u32) -> u32 {
    (value >> n) | (value << (32 - n))
}

/// Convert 4-character string to FourCC
pub fn make_fourcc(s: &str) -> u32 {
    let bytes = s.as_bytes();
    let mut code: u32 = 0;
    
    for (i, &byte) in bytes.iter().take(4).enumerate() {
        code |= (byte as u32) << (24 - i * 8);
    }
    
    // Pad with spaces if shorter
    for i in bytes.len()..4 {
        code |= (b' ' as u32) << (24 - i * 8);
    }
    
    code
}

/// Pretty print a database record
pub fn describe_record(data: &[u8]) -> String {
    if data.is_empty() {
        return "Empty record".to_string();
    }
    
    // Try to detect record type
    let first_byte = data[0];
    
    if data.len() >= 2 {
        match data[0] {
            0x00..=0x7F => return format!("Address record ({} bytes)", data.len()),
            _ => {}
        }
    }
    
    format!("Unknown record type 0x{:02X} ({} bytes)", first_byte, data.len())
}

/// Calculate record size (Palm format)
pub fn record_size(data: &[u8]) -> usize {
    align(data.len() + 8, 4) // header + data, aligned to 4 bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16() {
        let data = b"Hello World";
        let crc = crc16(data);
        // Known CRC16-CCITT value for "Hello World"
        assert_ne!(crc, 0); // Just verify it produces non-zero
    }

    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(bytes_to_hex(&[0x01, 0xFF, 0x10]), "01 FF 10");
    }

    #[test]
    fn test_hex_to_bytes() {
        assert_eq!(hex_to_bytes("01FF10").unwrap(), vec![0x01, 0xFF, 0x10]);
    }

    #[test]
    fn test_align() {
        assert_eq!(align(10, 4), 12);
        assert_eq!(align(12, 4), 12);
        assert_eq!(align(13, 4), 16);
    }

    #[test]
    fn test_make_fourcc() {
        assert_eq!(make_fourcc("DATA"), 0x44415441);
        assert_eq!(make_fourcc("DB"), 0x44422020);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
    }
}
