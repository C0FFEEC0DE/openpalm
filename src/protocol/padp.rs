//! Palm Access Data Protocol (PADP) implementation
//!
//! PADP is the protocol that sits between DLP and SLP in the Palm communication stack.
//! It handles packet fragmentation, reassembly, and reliable delivery.

use crate::error::{PilotError, Result};
use std::io::{Read, Write};

// PADP Constants
const PADP_HEADER_LEN: usize = 5;
const PADP_HEADER_LONG_LEN: usize = 7;
const PADP_MTU: usize = 1024;
const PADP_TX_TIMEOUT_MS: u64 = 2000;
const PADP_TX_RETRIES: u32 = 10;

/// PADP packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadpType {
    /// Data packet
    Data = 0x00,
    /// Acknowledgment
    Ack = 0x01,
    /// Keepalive/tickle
    Tick = 0x02,
    /// Wake up
    Wake = 0x03,
}

impl PadpType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val & 0x03 {
            0x00 => Some(PadpType::Data),
            0x01 => Some(PadpType::Ack),
            0x02 => Some(PadpType::Tick),
            0x03 => Some(PadpType::Wake),
            _ => None,
        }
    }
}

/// PADP flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PadpFlags(u8);

impl std::ops::BitOr for PadpFlags {
    type Output = Self;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        PadpFlags(self.0 | rhs.0)
    }
}

impl PadpFlags {
    /// First packet in sequence
    pub const FIRST: PadpFlags = PadpFlags(0x80);
    /// Last packet in sequence
    pub const LAST: PadpFlags = PadpFlags(0x40);
    /// Use long format (32-bit size)
    pub const LONG: PadpFlags = PadpFlags(0x20);
    /// Memory error flag
    pub const MEM_ERROR: PadpFlags = PadpFlags(0x10);
    
    /// Create empty flags
    pub fn empty() -> Self {
        PadpFlags(0)
    }
    
    /// Create flags from bits
    pub fn from_bits(bits: u8) -> Option<Self> {
        Some(PadpFlags(bits))
    }
    
    /// Get bits value
    pub fn bits(&self) -> u8 {
        self.0
    }
    
    /// Check if contains a flag
    pub fn contains(&self, flag: PadpFlags) -> bool {
        (self.0 & flag.0) == flag.0
    }
    
    /// Insert a flag
    pub fn insert(&mut self, flag: PadpFlags) {
        self.0 |= flag.0;
    }
    
    /// Remove a flag
    pub fn remove(&mut self, flag: PadpFlags) {
        self.0 &= !flag.0;
    }
}

/// PADP packet
#[derive(Debug, Clone)]
pub struct PadpPacket {
    /// Packet type
    pub packet_type: PadpType,
    /// Packet flags
    pub flags: PadpFlags,
    /// Transaction ID
    pub txid: u8,
    /// Packet size (or total size if FIRST flag)
    pub size: u32,
    /// Packet data
    pub data: Vec<u8>,
}

impl PadpPacket {
    /// Create a new data packet
    pub fn new_data(txid: u8, flags: PadpFlags, size: u32, data: Vec<u8>) -> Self {
        Self {
            packet_type: PadpType::Data,
            flags,
            txid,
            size,
            data,
        }
    }
    
    /// Create an ACK packet
    pub fn ack(txid: u8) -> Self {
        Self {
            packet_type: PadpType::Ack,
            flags: PadpFlags::empty(),
            txid,
            size: 0,
            data: Vec::new(),
        }
    }
    
    /// Create a tickle packet
    pub fn tickle(txid: u8) -> Self {
        Self {
            packet_type: PadpType::Tick,
            flags: PadpFlags::empty(),
            txid,
            size: 0,
            data: Vec::new(),
        }
    }
    
    /// Create a wake packet
    pub fn wake() -> Self {
        Self {
            packet_type: PadpType::Wake,
            flags: PadpFlags::empty(),
            txid: 0xFF,
            size: 0,
            data: Vec::new(),
        }
    }
    
    /// Encode packet to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(PADP_HEADER_LEN + self.data.len());
        
        // Type byte
        result.push(self.packet_type as u8);
        
        // Flags byte
        result.push(self.flags.bits());
        
        // TXID
        result.push(self.txid);
        
        // Size (use long format if needed)
        if self.flags.contains(PadpFlags::LONG) || self.size > 0xFFFF {
            result.push((self.size >> 24) as u8);
            result.push((self.size >> 16) as u8);
            result.push((self.size >> 8) as u8);
            result.push(self.size as u8);
        } else {
            result.push((self.size >> 8) as u8);
            result.push(self.size as u8);
        }
        
        // Data
        result.extend_from_slice(&self.data);
        
        result
    }
    
    /// Decode packet from bytes
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < PADP_HEADER_LEN {
            return Err(PilotError::ProtBadPacket);
        }
        
        let packet_type = PadpType::from_u8(data[0])
            .ok_or(PilotError::ProtBadPacket)?;
        
        let flags = PadpFlags::from_bits(data[1])
            .ok_or(PilotError::ProtBadPacket)?;
        
        let txid = data[2];
        
        let size = if flags.contains(PadpFlags::LONG) {
            if data.len() < PADP_HEADER_LONG_LEN {
                return Err(PilotError::ProtBadPacket);
            }
            u32::from_be_bytes([data[3], data[4], data[5], data[6]])
        } else {
            u16::from_be_bytes([data[3], data[4]]) as u32
        };
        
        let packet_data = if flags.contains(PadpFlags::LONG) {
            &data[PADP_HEADER_LONG_LEN..]
        } else {
            &data[PADP_HEADER_LEN..]
        };
        
        Ok(Self {
            packet_type,
            flags,
            txid,
            size,
            data: packet_data.to_vec(),
        })
    }
}

/// PADP state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadpState {
    Idle,
    Sending,
    Receiving,
    Waiting,
}

/// PADP connection handler
pub struct PadpConnection<S: Read + Write + Send> {
    /// Current state
    state: PadpState,
    /// Transaction ID for sending
    txid: u8,
    /// Next transaction ID
    next_txid: u8,
    /// Freeze TXID flag
    freeze_txid: bool,
    /// Use long format
    use_long_format: bool,
    /// Receive buffer
    recv_buffer: Vec<u8>,
    /// Receive TXID (last seen)
    recv_txid: u8,
    /// Stream reference
    stream: Option<S>,
}

impl<S: Read + Write + Send> PadpConnection<S> {
    /// Create new PADP connection
    pub fn new(stream: S) -> Self {
        Self {
            state: PadpState::Idle,
            txid: 0xFF,
            next_txid: 0x10,
            freeze_txid: false,
            use_long_format: false,
            recv_buffer: Vec::new(),
            recv_txid: 0,
            stream: Some(stream),
        }
    }
    
    /// Wake up the connection
    pub fn wake(&mut self) -> Result<()> {
        let stream = self.stream.as_mut()
            .ok_or(PilotError::SockDisconnected)?;
        
        let packet = PadpPacket::wake();
        stream.write_all(&packet.encode())
            .map_err(|_| PilotError::SockIo)?;
        stream.flush().map_err(|_| PilotError::SockIo)?;
        
        self.txid = 0xFF;
        Ok(())
    }
    
    /// Send data with reliable delivery
    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        let total_len = data.len();
        let mut offset = 0;
        let mut first = true;
        
        while offset < data.len() {
            let chunk_size = std::cmp::min(PADP_MTU, data.len() - offset);
            let is_last = offset + chunk_size >= data.len();
            
            let mut flags = PadpFlags::empty();
            if first {
                flags.insert(PadpFlags::FIRST);
            }
            if is_last {
                flags.insert(PadpFlags::LAST);
            }
            if self.use_long_format {
                flags.insert(PadpFlags::LONG);
            }
            
            let size_hint = if first { total_len as u32 } else { offset as u32 };
            
            let packet = PadpPacket::new_data(
                self.txid,
                flags,
                size_hint,
                data[offset..offset + chunk_size].to_vec(),
            );
            
            // Send packet
            {
                let stream = self.stream.as_mut()
                    .ok_or(PilotError::SockDisconnected)?;
                stream.write_all(&packet.encode())
                    .map_err(|_| PilotError::SockIo)?;
                stream.flush().map_err(|_| PilotError::SockIo)?;
            }
            
            // Wait for ACK
            if packet.packet_type != PadpType::Tick {
                let response = self.receive_packet()?;
                
                match response.packet_type {
                    PadpType::Ack if response.txid == self.txid => {
                        // Success
                    }
                    PadpType::Tick => {
                        // Keep waiting
                        continue;
                    }
                    PadpType::Ack => {
                        return Err(PilotError::ProtBadPacket);
                    }
                    _ => {
                        return Err(PilotError::ProtBadPacket);
                    }
                }
            }
            
            offset += chunk_size;
            first = false;
            self.txid = self.next_txid;
        }
        
        Ok(total_len)
    }
    
    /// Receive a packet
    fn receive_packet(&mut self) -> Result<PadpPacket> {
        use std::io::Read;
        
        let stream = self.stream.as_mut()
            .ok_or(PilotError::SockDisconnected)?;
        
        let mut header = [0u8; PADP_HEADER_LEN + 2];
        
        // Read header
        let mut pos = 0;
        while pos < PADP_HEADER_LEN + 2 {
            let n = stream.read(&mut header[pos..])
                .map_err(|_| PilotError::SockIo)?;
            if n == 0 {
                return Err(PilotError::SockDisconnected);
            }
            pos += n;
        }
        
        let packet_type = PadpType::from_u8(header[0])
            .ok_or(PilotError::ProtBadPacket)?;
        
        let flags = PadpFlags::from_bits(header[1])
            .ok_or(PilotError::ProtBadPacket)?;
        
        let txid = header[2];
        
        let size = if flags.contains(PadpFlags::LONG) {
            u32::from_be_bytes([header[3], header[4], header[5], header[6]])
        } else {
            u16::from_be_bytes([header[3], header[4]]) as u32
        };
        
        // Read data
        let data_len = if flags.contains(PadpFlags::LONG) {
            size as usize
        } else {
            std::cmp::min(size as usize, PADP_MTU)
        };
        
        let mut data = vec![0u8; data_len];
        let mut received = 0;
        while received < data_len {
            let n = stream.read(&mut data[received..])
                .map_err(|_| PilotError::SockIo)?;
            if n == 0 {
                break;
            }
            received += n;
        }
        
        let packet = PadpPacket {
            packet_type,
            flags,
            txid,
            size,
            data,
        };
        
        // Send ACK for data packets
        if packet.packet_type == PadpType::Data {
            let ack = PadpPacket::ack(packet.txid);
            stream.write_all(&ack.encode())
                .map_err(|_| PilotError::SockIo)?;
            stream.flush().map_err(|_| PilotError::SockIo)?;
        }
        
        Ok(packet)
    }
    
    /// Receive data (assembles fragments)
    pub fn receive(&mut self, buffer: &mut [u8]) -> Result<usize> {
        loop {
            let packet = self.receive_packet()?;
            
            match packet.packet_type {
                PadpType::Data => {
                    // Check if this is a new transaction
                    if packet.flags.contains(PadpFlags::FIRST) {
                        self.recv_buffer.clear();
                        self.recv_txid = packet.txid;
                    }
                    
                    // Verify TXID matches
                    if packet.txid != self.recv_txid {
                        continue;
                    }
                    
                    // Append data
                    self.recv_buffer.extend_from_slice(&packet.data);
                    
                    // Check if complete
                    if packet.flags.contains(PadpFlags::LAST) {
                        let len = std::cmp::min(buffer.len(), self.recv_buffer.len());
                        buffer[..len].copy_from_slice(&self.recv_buffer[..len]);
                        return Ok(len);
                    }
                }
                PadpType::Tick => {
                    // Just ignore tickles, stay in waiting state
                }
                PadpType::Wake => {
                    self.txid = 0xFF;
                }
                _ => {}
            }
        }
    }
    
    /// Get connection state
    pub fn state(&self) -> PadpState {
        self.state
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
    
    /// Set long format usage
    pub fn set_long_format(&mut self, enabled: bool) {
        self.use_long_format = enabled;
    }
    
    /// Freeze/unfreeze TXID
    pub fn set_freeze_txid(&mut self, freeze: bool) {
        self.freeze_txid = freeze;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padp_type_from_u8() {
        assert_eq!(PadpType::from_u8(0x00), Some(PadpType::Data));
        assert_eq!(PadpType::from_u8(0x01), Some(PadpType::Ack));
        assert_eq!(PadpType::from_u8(0x02), Some(PadpType::Tick));
        assert_eq!(PadpType::from_u8(0x03), Some(PadpType::Wake));
    }

    #[test]
    fn test_padp_ack_packet() {
        let ack = PadpPacket::ack(0x42);
        assert_eq!(ack.packet_type, PadpType::Ack);
        assert_eq!(ack.txid, 0x42);
        assert!(ack.data.is_empty());
    }

    #[test]
    fn test_padp_wake_packet() {
        let wake = PadpPacket::wake();
        assert_eq!(wake.packet_type, PadpType::Wake);
        assert_eq!(wake.txid, 0xFF);
    }

    #[test]
    fn test_padp_encode_decode() {
        let original = PadpPacket::new_data(
            0x10,
            PadpFlags::FIRST | PadpFlags::LAST,
            4,
            vec![0x01, 0x02, 0x03, 0x04],
        );
        
        let encoded = original.encode();
        let decoded = PadpPacket::decode(&encoded).unwrap();
        
        assert_eq!(decoded.packet_type, PadpType::Data);
        assert_eq!(decoded.txid, 0x10);
        assert!(decoded.flags.contains(PadpFlags::FIRST));
        assert!(decoded.flags.contains(PadpFlags::LAST));
        assert_eq!(decoded.data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_padp_flags() {
        let mut flags = PadpFlags::empty();
        flags.insert(PadpFlags::FIRST);
        flags.insert(PadpFlags::LAST);
        
        assert!(flags.contains(PadpFlags::FIRST));
        assert!(flags.contains(PadpFlags::LAST));
        assert!(!flags.contains(PadpFlags::LONG));
    }
}
