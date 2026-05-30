//! System packets for Palm OS
//!
//! This module implements system packet handling for the Palm protocol stack.

use crate::error::{PilotError, Result};

/// System packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SysPktType {
    /// Request packet
    Request = 0x00,
    /// Response packet
    Response = 0x01,
    /// Data packet
    Data = 0x02,
    /// NAK (negative acknowledgment)
    Nak = 0x03,
    /// Abort packet
    Abort = 0x04,
    /// Heartbeat packet
    Heartbeat = 0x05,
}

impl SysPktType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(SysPktType::Request),
            1 => Some(SysPktType::Response),
            2 => Some(SysPktType::Data),
            3 => Some(SysPktType::Nak),
            4 => Some(SysPktType::Abort),
            5 => Some(SysPktType::Heartbeat),
            _ => None,
        }
    }
}

/// System packet commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SysPktCmd {
    /// Get system info
    GetSysInfo = 0x00,
    /// Get ROM version
    GetRomVersion = 0x01,
    /// Get user info
    GetUserInfo = 0x02,
    /// System reset
    Reset = 0x03,
    /// Wake up
    Wake = 0x04,
    /// Get last error
    GetLastError = 0x05,
    /// Register
    Register = 0x06,
    /// Keep alive
    KeepAlive = 0x07,
}

impl SysPktCmd {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(SysPktCmd::GetSysInfo),
            1 => Some(SysPktCmd::GetRomVersion),
            2 => Some(SysPktCmd::GetUserInfo),
            3 => Some(SysPktCmd::Reset),
            4 => Some(SysPktCmd::Wake),
            5 => Some(SysPktCmd::GetLastError),
            6 => Some(SysPktCmd::Register),
            7 => Some(SysPktCmd::KeepAlive),
            _ => None,
        }
    }
}

/// System packet
#[derive(Debug, Clone)]
pub struct SysPkt {
    /// Packet type
    pub pkt_type: SysPktType,
    /// Command
    pub cmd: SysPktCmd,
    /// Sequence number
    pub seq: u8,
    /// Flags
    pub flags: u8,
    /// Payload
    pub payload: Vec<u8>,
}

impl SysPkt {
    /// Create new packet
    pub fn new(pkt_type: SysPktType, cmd: SysPktCmd, seq: u8) -> Self {
        Self {
            pkt_type,
            cmd,
            seq,
            flags: 0,
            payload: Vec::new(),
        }
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Header: type (1) + cmd (1) + seq (1) + flags (1) + length (2)
        bytes.push(self.pkt_type as u8);
        bytes.push(self.cmd as u8);
        bytes.push(self.seq);
        bytes.push(self.flags);
        bytes.extend_from_slice(&(self.payload.len() as u16).to_be_bytes());

        // Payload
        bytes.extend_from_slice(&self.payload);

        bytes
    }

    /// Parse from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 6 {
            return Err(PilotError::InvalidData("SysPkt too short".into()));
        }

        let pkt_type = SysPktType::from_u8(data[0])
            .ok_or_else(|| PilotError::InvalidData("Invalid SysPkt type".into()))?;

        let cmd = SysPktCmd::from_u8(data[1])
            .ok_or_else(|| PilotError::InvalidData("Invalid SysPkt cmd".into()))?;

        let seq = data[2];
        let flags = data[3];
        let length = u16::from_be_bytes([data[4], data[5]]) as usize;

        if data.len() < 6 + length {
            return Err(PilotError::InvalidData("SysPkt length mismatch".into()));
        }

        let payload = data[6..6 + length].to_vec();

        Ok(Self {
            pkt_type,
            cmd,
            seq,
            flags,
            payload,
        })
    }

    /// Create request packet
    pub fn request(cmd: SysPktCmd, seq: u8) -> Self {
        Self::new(SysPktType::Request, cmd, seq)
    }

    /// Create response packet
    pub fn response(cmd: SysPktCmd, seq: u8) -> Self {
        Self::new(SysPktType::Response, cmd, seq)
    }

    /// Create NAK packet
    pub fn nak(seq: u8) -> Self {
        Self::new(SysPktType::Nak, SysPktCmd::GetSysInfo, seq)
    }

    /// Create heartbeat packet
    pub fn heartbeat(seq: u8) -> Self {
        Self::new(SysPktType::Heartbeat, SysPktCmd::KeepAlive, seq)
    }
}

/// System info response
#[derive(Debug, Clone)]
pub struct SysInfo {
    /// ROM version major
    pub rom_version_major: u8,
    /// ROM version minor
    pub rom_version_minor: u8,
    /// ROM version dot
    pub rom_version_dot: u8,
    /// Locale
    pub locale: [u8; 4],
    /// Device ID
    pub device_id: u32,
    /// Product ID
    pub product_id: u32,
    /// Serial number
    pub serial: [u8; 12],
}

impl SysInfo {
    /// Parse from payload
    pub fn parse(payload: &[u8]) -> Result<Self> {
        if payload.len() < 27 {
            return Err(PilotError::InvalidData("SysInfo payload too short".into()));
        }

        Ok(Self {
            rom_version_major: payload[0],
            rom_version_minor: payload[1],
            rom_version_dot: payload[2],
            locale: [payload[3], payload[4], payload[5], payload[6]],
            device_id: u32::from_be_bytes([payload[7], payload[8], payload[9], payload[10]]),
            product_id: u32::from_be_bytes([payload[11], payload[12], payload[13], payload[14]]),
            serial: [
                payload[15],
                payload[16],
                payload[17],
                payload[18],
                payload[19],
                payload[20],
                payload[21],
                payload[22],
                payload[23],
                payload[24],
                payload[25],
                payload[26],
            ],
        })
    }

    /// Pack to payload
    pub fn pack(&self) -> Vec<u8> {
        let mut payload = Vec::new();

        payload.push(self.rom_version_major);
        payload.push(self.rom_version_minor);
        payload.push(self.rom_version_dot);
        payload.extend_from_slice(&self.locale);
        payload.extend_from_slice(&self.device_id.to_be_bytes());
        payload.extend_from_slice(&self.product_id.to_be_bytes());
        payload.extend_from_slice(&self.serial);

        payload
    }
}

/// User info
#[derive(Debug, Clone)]
pub struct UserInfo {
    /// User ID
    pub user_id: u32,
    /// User name
    pub user_name: String,
    /// User password (hashed)
    pub password_hash: [u8; 16],
}

impl UserInfo {
    /// Parse from payload
    pub fn parse(payload: &[u8]) -> Result<Self> {
        if payload.len() < 24 {
            return Err(PilotError::InvalidData("UserInfo payload too short".into()));
        }

        let user_id = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);

        // Parse null-terminated string
        let mut name_end = 4;
        while name_end < payload.len() && payload[name_end] != 0 {
            name_end += 1;
        }
        let user_name = String::from_utf8_lossy(&payload[4..name_end]).to_string();

        let password_hash = [
            payload[name_end + 1],
            payload[name_end + 2],
            payload[name_end + 3],
            payload[name_end + 4],
            payload[name_end + 5],
            payload[name_end + 6],
            payload[name_end + 7],
            payload[name_end + 8],
            payload[name_end + 9],
            payload[name_end + 10],
            payload[name_end + 11],
            payload[name_end + 12],
            payload[name_end + 13],
            payload[name_end + 14],
            payload[name_end + 15],
            payload[name_end + 16],
        ];

        Ok(Self {
            user_id,
            user_name,
            password_hash,
        })
    }
}

/// System packet handler
pub struct SysPktHandler {
    /// Next sequence number
    next_seq: u8,
    /// Timeout (ms)
    timeout_ms: u32,
    /// Max retries
    max_retries: u8,
}

impl SysPktHandler {
    /// Create new handler
    pub fn new() -> Self {
        Self {
            next_seq: 0,
            timeout_ms: 5000,
            max_retries: 3,
        }
    }

    /// Get next sequence number
    pub fn next_seq(&mut self) -> u8 {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);
        seq
    }

    /// Set timeout
    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    /// Set max retries
    pub fn set_max_retries(&mut self, max: u8) {
        self.max_retries = max;
    }

    /// Create GetSysInfo request
    pub fn get_sys_info(&mut self) -> SysPkt {
        SysPkt::request(SysPktCmd::GetSysInfo, self.next_seq())
    }

    /// Create GetRomVersion request
    pub fn get_rom_version(&mut self) -> SysPkt {
        SysPkt::request(SysPktCmd::GetRomVersion, self.next_seq())
    }

    /// Create GetUserInfo request
    pub fn get_user_info(&mut self) -> SysPkt {
        SysPkt::request(SysPktCmd::GetUserInfo, self.next_seq())
    }

    /// Create Reset request
    pub fn reset(&mut self) -> SysPkt {
        SysPkt::request(SysPktCmd::Reset, self.next_seq())
    }

    /// Create Wake request
    pub fn wake(&mut self) -> SysPkt {
        SysPkt::request(SysPktCmd::Wake, self.next_seq())
    }

    /// Handle incoming packet
    pub fn handle(&mut self, pkt: &SysPkt) -> Option<SysPkt> {
        match pkt.pkt_type {
            SysPktType::Request => {
                // Respond with appropriate response
                match pkt.cmd {
                    SysPktCmd::GetSysInfo => {
                        // In real implementation, would gather actual info
                        let mut resp = SysPkt::response(pkt.cmd, pkt.seq);
                        resp.payload = vec![0; 27]; // Placeholder
                        Some(resp)
                    }
                    SysPktCmd::GetRomVersion => {
                        let mut resp = SysPkt::response(pkt.cmd, pkt.seq);
                        resp.payload = vec![5, 0, 0]; // Example version
                        Some(resp)
                    }
                    SysPktCmd::GetUserInfo => {
                        let mut resp = SysPkt::response(pkt.cmd, pkt.seq);
                        resp.payload = vec![1, 0, 0, 0]; // Example user ID
                        Some(resp)
                    }
                    SysPktCmd::Reset | SysPktCmd::Wake => {
                        // Just acknowledge
                        Some(SysPkt::response(pkt.cmd, pkt.seq))
                    }
                    _ => None,
                }
            }
            SysPktType::Nak => {
                // Retransmission needed
                None
            }
            _ => None,
        }
    }
}

impl Default for SysPktHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// SysPkt constants
pub mod constants {
    /// Maximum payload size
    pub const MAX_PAYLOAD: usize = 65535;

    /// Default timeout (ms)
    pub const DEFAULT_TIMEOUT: u32 = 5000;

    /// Default max retries
    pub const DEFAULT_MAX_RETRIES: u8 = 3;

    /// Heartbeat interval (ms)
    pub const HEARTBEAT_INTERVAL: u32 = 30000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_pkt_pack_parse() {
        let pkt = SysPkt::request(SysPktCmd::GetSysInfo, 42);
        let bytes = pkt.pack();
        let parsed = SysPkt::parse(&bytes).unwrap();

        assert_eq!(parsed.pkt_type, SysPktType::Request);
        assert_eq!(parsed.cmd, SysPktCmd::GetSysInfo);
        assert_eq!(parsed.seq, 42);
    }

    #[test]
    fn test_sys_pkt_type() {
        assert_eq!(SysPktType::from_u8(0), Some(SysPktType::Request));
        assert_eq!(SysPktType::from_u8(5), Some(SysPktType::Heartbeat));
        assert_eq!(SysPktType::from_u8(6), None);
    }

    #[test]
    fn test_sys_pkt_cmd() {
        assert_eq!(SysPktCmd::from_u8(0), Some(SysPktCmd::GetSysInfo));
        assert_eq!(SysPktCmd::from_u8(7), Some(SysPktCmd::KeepAlive));
        assert_eq!(SysPktCmd::from_u8(8), None);
    }

    #[test]
    fn test_sys_info_pack_parse() {
        let info = SysInfo {
            rom_version_major: 5,
            rom_version_minor: 4,
            rom_version_dot: 2,
            locale: *b"enUS",
            device_id: 0x12345678,
            product_id: 0x87654321,
            serial: *b"ABC123456789",
        };

        let payload = info.pack();
        let parsed = SysInfo::parse(&payload).unwrap();

        assert_eq!(parsed.rom_version_major, 5);
        assert_eq!(parsed.device_id, 0x12345678);
    }

    #[test]
    fn test_nak_heartbeat() {
        let nak = SysPkt::nak(5);
        assert_eq!(nak.pkt_type, SysPktType::Nak);
        assert_eq!(nak.seq, 5);

        let heartbeat = SysPkt::heartbeat(10);
        assert_eq!(heartbeat.pkt_type, SysPktType::Heartbeat);
        assert_eq!(heartbeat.seq, 10);
    }
}
