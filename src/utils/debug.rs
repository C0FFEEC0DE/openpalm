//! Debug utilities for openpalm
//!
//! This module provides debugging and logging utilities.

use std::fmt;

/// Debug level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DebugLevel {
    None = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Debug = 4,
    Verbose = 5,
}

impl DebugLevel {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => DebugLevel::None,
            1 => DebugLevel::Error,
            2 => DebugLevel::Warning,
            3 => DebugLevel::Info,
            4 => DebugLevel::Debug,
            _ => DebugLevel::Verbose,
        }
    }
}

/// Hex dump options
#[derive(Debug, Clone)]
pub struct HexDumpOptions {
    /// Bytes per line
    pub bytes_per_line: usize,
    /// Show ASCII column
    pub show_ascii: bool,
    /// Show offset column
    pub show_offset: bool,
    /// Group size (bytes)
    pub group_size: usize,
}

impl Default for HexDumpOptions {
    fn default() -> Self {
        Self {
            bytes_per_line: 16,
            show_ascii: true,
            show_offset: true,
            group_size: 1,
        }
    }
}

impl HexDumpOptions {
    /// Standard hex dump (16 bytes per line)
    pub fn standard() -> Self {
        Self::default()
    }
    
    /// Compact hex dump (8 bytes per line)
    pub fn compact() -> Self {
        Self {
            bytes_per_line: 8,
            show_ascii: false,
            show_offset: false,
            group_size: 1,
        }
    }
    
    /// Detailed hex dump with 4-byte groups
    pub fn detailed() -> Self {
        Self {
            bytes_per_line: 16,
            show_ascii: true,
            show_offset: true,
            group_size: 4,
        }
    }
}

/// Create a hex dump of data
pub fn hex_dump(data: &[u8], options: &HexDumpOptions) -> String {
    let mut result = String::new();
    let mut offset = 0;
    
    while offset < data.len() {
        let end = std::cmp::min(offset + options.bytes_per_line, data.len());
        let line = &data[offset..end];
        
        // Offset
        if options.show_offset {
            result.push_str(&format!("{:04X}: ", offset));
        }
        
        // Hex bytes
        let mut hex_str = String::new();
        for (i, &byte) in line.iter().enumerate() {
            hex_str.push_str(&format!("{:02X}", byte));
            if options.group_size > 1 && (i + 1) % options.group_size == 0 && i < line.len() - 1 {
                hex_str.push(' ');
            } else {
                hex_str.push(' ');
            }
        }
        
        // Pad hex if needed
        while hex_str.len() < options.bytes_per_line * 3 {
            hex_str.push(' ');
        }
        
        result.push_str(&hex_str);
        
        // ASCII representation
        if options.show_ascii {
            result.push_str(" |");
            for &byte in line {
                if byte.is_ascii_graphic() || byte == b' ' {
                    result.push(byte as char);
                } else {
                    result.push('.');
                }
            }
            result.push('|');
        }
        
        result.push('\n');
        offset = end;
    }
    
    result
}

/// Dump a protocol packet
pub fn dump_packet(packet_type: &str, data: &[u8]) -> String {
    let mut result = format!("=== {} ({} bytes) ===\n", packet_type, data.len());
    result.push_str(&hex_dump(data, &HexDumpOptions::standard()));
    result
}

/// Dump SLP packet
pub fn dump_slp_packet(data: &[u8]) -> String {
    if data.is_empty() {
        return "Empty SLP packet".to_string();
    }
    
    let mut result = String::from("SLP Packet:\n");
    
    // Try to parse header
    if data.len() >= 4 {
        result.push_str(&format!("  Type: 0x{:02X}\n", data[0]));
        result.push_str(&format!("  Flags: 0x{:02X}\n", data[1]));
        result.push_str(&format!("  Seq: {}\n", data[2]));
        result.push_str(&format!("  Len: {}\n", u16::from_be_bytes([data[3], data[4]])));
    }
    
    result.push_str("  Data:\n");
    result.push_str(&hex_dump(data, &HexDumpOptions::compact()));
    
    result
}

/// Dump PADP packet
pub fn dump_padp_packet(data: &[u8]) -> String {
    if data.is_empty() {
        return "Empty PADP packet".to_string();
    }
    
    let mut result = String::from("PADP Packet:\n");
    
    if data.len() >= 4 {
        result.push_str(&format!("  Type: 0x{:02X}\n", data[0] & 0x03));
        result.push_str(&format!("  Flags: 0x{:02X}\n", data[1]));
        result.push_str(&format!("  TXID: {}\n", data[2]));
        let len = u16::from_be_bytes([data[3], data[4]]);
        result.push_str(&format!("  Size: {}\n", len));
    }
    
    result.push('\n');
    result.push_str(&hex_dump(data, &HexDumpOptions::standard()));
    
    result
}

/// Dump DLP packet
pub fn dump_dlp_packet(data: &[u8]) -> String {
    if data.len() < 6 {
        return format!("DLP packet too short ({} bytes)", data.len());
    }
    
    let mut result = String::from("DLP Packet:\n");
    
    let command = u16::from_be_bytes([data[0], data[1]]);
    let arg_size = u16::from_be_bytes([data[2], data[3]]);
    let flags = data[4];
    
    result.push_str(&format!("  Command: 0x{:04X}\n", command));
    result.push_str(&format!("  Arg Size: {}\n", arg_size));
    result.push_str(&format!("  Flags: 0x{:02X}\n", flags));
    
    if data.len() > 6 {
        result.push_str("  Args:\n");
        result.push_str(&hex_dump(&data[6..], &HexDumpOptions::compact()));
    }
    
    result
}

/// Logger
pub struct Logger {
    level: DebugLevel,
    prefix: String,
}

impl Logger {
    /// Create a new logger
    pub fn new(level: DebugLevel, prefix: &str) -> Self {
        Self {
            level,
            prefix: prefix.to_string(),
        }
    }
    
    /// Set debug level
    pub fn set_level(&mut self, level: DebugLevel) {
        self.level = level;
    }
    
    /// Log error
    pub fn error(&self, msg: &str) {
        if self.level >= DebugLevel::Error {
            eprintln!("[{} ERROR] {}", self.prefix, msg);
        }
    }
    
    /// Log warning
    pub fn warning(&self, msg: &str) {
        if self.level >= DebugLevel::Warning {
            eprintln!("[{} WARN] {}", self.prefix, msg);
        }
    }
    
    /// Log info
    pub fn info(&self, msg: &str) {
        if self.level >= DebugLevel::Info {
            println!("[{} INFO] {}", self.prefix, msg);
        }
    }
    
    /// Log debug
    pub fn debug(&self, msg: &str) {
        if self.level >= DebugLevel::Debug {
            println!("[{} DEBUG] {}", self.prefix, msg);
        }
    }
    
    /// Log verbose
    pub fn verbose(&self, msg: &str) {
        if self.level >= DebugLevel::Verbose {
            println!("[{} VERBOSE] {}", self.prefix, msg);
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(DebugLevel::Warning, "openpalm")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_dump_options() {
        let opts = HexDumpOptions::standard();
        assert_eq!(opts.bytes_per_line, 16);
        assert!(opts.show_ascii);
        assert!(opts.show_offset);
    }

    #[test]
    fn test_hex_dump() {
        let data = b"Hello, World!";
        let dump = hex_dump(data, &HexDumpOptions::standard());
        assert!(dump.contains("Hello"));
    }

    #[test]
    fn test_dump_packet() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let dump = dump_packet("TEST", &data);
        assert!(dump.contains("TEST"));
        assert!(dump.contains("4 bytes"));
    }
}
