//! Serial Link Protocol (SLP) implementation
//!
//! SLP is the lowest-level protocol in the Palm communication stack.
//! It provides reliable byte-stream communication over serial/USB connections.

use crate::error::{PilotError, Result};
use std::io::{Read, Write};

// SLP Constants
const SLP_FLAG_SLIP_MODE: u8 = 0x01;
const SLP_FLAG_COMPRESSED: u8 = 0x02;
const SLP_FLAG_ENCRYPTED: u8 = 0x04;
const SLP_FLAG_CHECKSUM: u8 = 0x08;

/// SLP packet types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlpPacketType {
    Data = 0x00,
    Ack = 0x01,
    Nak = 0x02,
    Cancel = 0x03,
    Nack = 0x04,
    Ping = 0x05,
    Pong = 0x06,
    Reset = 0x07,
    Handshake = 0x08,
    HandshakeAck = 0x09,
    Sync = 0x0A,
    SyncAck = 0x0B,
}

impl SlpPacketType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val & 0x0F {
            0x00 => Some(SlpPacketType::Data),
            0x01 => Some(SlpPacketType::Ack),
            0x02 => Some(SlpPacketType::Nak),
            0x03 => Some(SlpPacketType::Cancel),
            0x04 => Some(SlpPacketType::Nack),
            0x05 => Some(SlpPacketType::Ping),
            0x06 => Some(SlpPacketType::Pong),
            0x07 => Some(SlpPacketType::Reset),
            0x08 => Some(SlpPacketType::Handshake),
            0x09 => Some(SlpPacketType::HandshakeAck),
            0x0A => Some(SlpPacketType::Sync),
            0x0B => Some(SlpPacketType::SyncAck),
            _ => None,
        }
    }
}

/// SLP flags
#[derive(Debug, Clone, Default)]
pub struct SlpFlags(u8);

impl SlpFlags {
    /// Create new flags with default values
    pub fn new() -> Self {
        Self(0)
    }
    
    /// Create flags from raw byte value
    pub fn from_u8(val: u8) -> Self {
        Self(val)
    }
    
    /// Get raw byte value
    pub fn value(&self) -> u8 {
        self.0
    }
    
    /// Check if SLIP mode is enabled
    pub fn slip_mode(&self) -> bool { (self.0 & SLP_FLAG_SLIP_MODE) != 0 }
    
    /// Check if compression is enabled
    pub fn compressed(&self) -> bool { (self.0 & SLP_FLAG_COMPRESSED) != 0 }
    
    /// Check if encryption is enabled
    pub fn encrypted(&self) -> bool { (self.0 & SLP_FLAG_ENCRYPTED) != 0 }
    
    /// Check if checksum is enabled
    pub fn checksum(&self) -> bool { (self.0 & SLP_FLAG_CHECKSUM) != 0 }
    
    /// Set SLIP mode flag
    pub fn set_slip_mode(&mut self, enabled: bool) -> &mut Self {
        if enabled { 
            self.0 |= SLP_FLAG_SLIP_MODE; 
        } else { 
            self.0 &= !SLP_FLAG_SLIP_MODE; 
        }
        self
    }
    
    /// Set compressed flag
    pub fn set_compressed(&mut self, enabled: bool) -> &mut Self {
        if enabled { 
            self.0 |= SLP_FLAG_COMPRESSED; 
        } else { 
            self.0 &= !SLP_FLAG_COMPRESSED; 
        }
        self
    }
    
    /// Set encrypted flag
    pub fn set_encrypted(&mut self, enabled: bool) -> &mut Self {
        if enabled { 
            self.0 |= SLP_FLAG_ENCRYPTED; 
        } else { 
            self.0 &= !SLP_FLAG_ENCRYPTED; 
        }
        self
    }
    
    /// Set checksum flag
    pub fn set_checksum(&mut self, enabled: bool) -> &mut Self {
        if enabled { 
            self.0 |= SLP_FLAG_CHECKSUM; 
        } else { 
            self.0 &= !SLP_FLAG_CHECKSUM; 
        }
        self
    }
    
    /// Enable SLIP mode (builder pattern)
    pub fn with_slip_mode(mut self) -> Self { 
        self.0 |= SLP_FLAG_SLIP_MODE; 
        self 
    }
    
    /// Enable compression (builder pattern)
    pub fn with_compressed(mut self) -> Self { 
        self.0 |= SLP_FLAG_COMPRESSED; 
        self 
    }
    
    /// Enable encryption (builder pattern)
    pub fn with_encrypted(mut self) -> Self { 
        self.0 |= SLP_FLAG_ENCRYPTED; 
        self 
    }
    
    /// Enable checksum (builder pattern)
    pub fn with_checksum(mut self) -> Self { 
        self.0 |= SLP_FLAG_CHECKSUM; 
        self 
    }
}

/// SLP packet
#[derive(Debug, Clone)]
pub struct SlpPacket {
    pub packet_type: SlpPacketType,
    pub flags: SlpFlags,
    pub seq_num: u8,
    pub checksum: u8,
    pub data: Vec<u8>,
}

impl SlpPacket {
    /// Create a new packet
    pub fn new(packet_type: SlpPacketType, seq_num: u8, data: Vec<u8>) -> Self {
        Self {
            packet_type,
            flags: SlpFlags::default(),
            seq_num,
            checksum: 0,
            data,
        }
    }
    
    /// Create an ACK packet
    pub fn ack(seq_num: u8) -> Self {
        Self::new(SlpPacketType::Ack, seq_num, Vec::new())
    }
    
    /// Create a NAK packet
    pub fn nak(seq_num: u8) -> Self {
        Self::new(SlpPacketType::Nak, seq_num, Vec::new())
    }
    
    /// Create a data packet
    pub fn data(seq_num: u8, data: Vec<u8>) -> Self {
        Self::new(SlpPacketType::Data, seq_num, data)
    }
    
    /// Create a reset packet
    pub fn reset() -> Self {
        Self::new(SlpPacketType::Reset, 0, Vec::new())
    }
    
    /// Create a handshake packet
    pub fn handshake() -> Self {
        Self::new(SlpPacketType::Handshake, 0, vec![
            0x00, // Protocol version
            0x01, // Protocol minor
            0x00, // Reserved
            0x00, // Reserved
        ])
    }
    
    /// Encode packet to bytes (with SLIP escaping)
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.data.len() + 8);
        
        // Start marker
        result.push(0xC0);
        
        // Packet type and flags
        result.push((self.packet_type as u8) | (self.flags.0 << 4));
        
        // Sequence number
        result.push(self.seq_num);
        
        // Length (MSB, LSB)
        let len = self.data.len() as u16;
        result.push((len >> 8) as u8);
        result.push((len & 0xFF) as u8);
        
        // Data with SLIP escaping
        for &byte in &self.data {
            match byte {
                0xC0 => {
                    result.push(0xDB);
                    result.push(0xDC);
                }
                0xDB => {
                    result.push(0xDB);
                    result.push(0xDD);
                }
                _ => result.push(byte),
            }
        }
        
        // Checksum with SLIP escaping
        let checksum = self.calculate_checksum();
        match checksum {
            0xC0 => {
                result.push(0xDB);
                result.push(0xDC);
            }
            0xDB => {
                result.push(0xDB);
                result.push(0xDD);
            }
            _ => result.push(checksum),
        }

        // End marker
        result.push(0xC0);
        
        result
    }
    
    /// Decode packet from bytes (with SLIP unescaping)
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 6 {
            return Err(PilotError::ProtBadPacket);
        }
        
        // Skip start/end markers
        let inner = &data[1..data.len()-1];
        
        // Decode SLIP escaping
        let unescaped = Self::unescape(inner)?;
        
        if unescaped.len() < 5 {
            return Err(PilotError::ProtBadPacket);
        }
        
        let packet_type = SlpPacketType::from_u8(unescaped[0])
            .ok_or(PilotError::ProtBadPacket)?;
        
        let flags = SlpFlags(unescaped[0] >> 4);
        let seq_num = unescaped[1];
        let len = ((unescaped[2] as u16) << 8) | (unescaped[3] as u16);
        
        if unescaped.len() < 4 + len as usize + 1 {
            return Err(PilotError::ProtBadPacket);
        }
        
        let payload = &unescaped[4..4+len as usize];
        
        // Verify checksum
        let received_checksum = unescaped[4 + len as usize];
        let calculated = Self::calculate_checksum_raw(&unescaped[..4 + len as usize]);
        
        if received_checksum != calculated {
            return Err(PilotError::ProtBadPacket);
        }
        
        Ok(Self {
            packet_type,
            flags,
            seq_num,
            checksum: received_checksum,
            data: payload.to_vec(),
        })
    }
    
    /// SLIP unescape
    fn unescape(data: &[u8]) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(data.len());
        let mut i = 0;
        
        while i < data.len() {
            match data[i] {
                0xDB if i + 1 < data.len() => {
                    match data[i + 1] {
                        0xDC => { result.push(0xC0); i += 2; }
                        0xDD => { result.push(0xDB); i += 2; }
                        _ => { result.push(data[i]); i += 1; }
                    }
                }
                other => { result.push(other); i += 1; }
            }
        }
        
        Ok(result)
    }
    
    /// Calculate checksum for this packet
    fn calculate_checksum(&self) -> u8 {
        let mut header = vec![
            self.packet_type as u8 | (self.flags.0 << 4),
            self.seq_num,
            (self.data.len() >> 8) as u8,
            (self.data.len() & 0xFF) as u8,
        ];
        header.extend_from_slice(&self.data);
        Self::calculate_checksum_raw(&header)
    }
    
    /// Raw checksum calculation
    fn calculate_checksum_raw(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
    }
}

/// SLP State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlpState {
    Disconnected,
    Handshake,
    Syncing,
    Connected,
    Error,
}

/// SLP connection handler for sync I/O
pub struct SlpConnection<S: Read + Write + Send> {
    state: SlpState,
    seq_send: u8,
    seq_expect: u8,
    stream: Option<S>,
}

impl<S: Read + Write + Send> SlpConnection<S> {
    /// Create new SLP connection with a stream
    pub fn new(stream: S) -> Self {
        Self {
            state: SlpState::Disconnected,
            seq_send: 0,
            seq_expect: 0,
            stream: Some(stream),
        }
    }
    
    /// Connect and perform handshake
    pub fn connect(&mut self) -> Result<()> {
        let stream = self.stream.as_mut()
            .ok_or(PilotError::SockDisconnected)?;
        
        self.state = SlpState::Handshake;
        
        // Send handshake
        let handshake = SlpPacket::handshake();
        stream.write_all(&handshake.encode())
            .map_err(|_| PilotError::SockIo)?;
        stream.flush().map_err(|_| PilotError::SockIo)?;
        
        // Wait for handshake ack
        let _ = stream;
        let response = self.receive_packet()?;
        
        if response.packet_type == SlpPacketType::HandshakeAck {
            let stream = self.stream.as_mut()
                .ok_or(PilotError::SockDisconnected)?;
            self.state = SlpState::Syncing;
            
            // Send sync
            let sync = SlpPacket::new(SlpPacketType::Sync, 0, Vec::new());
            stream.write_all(&sync.encode())
                .map_err(|_| PilotError::SockIo)?;
            stream.flush().map_err(|_| PilotError::SockIo)?;
            let _ = stream;
            
            // Wait for sync ack
            let _response = self.receive_packet()?;
            
            self.state = SlpState::Connected;
            self.seq_send = 0;
            self.seq_expect = 0;
            Ok(())
        } else {
            self.state = SlpState::Error;
            Err(PilotError::ProtIncompatible)
        }
    }
    
    /// Disconnect
    pub fn disconnect(&mut self) -> Result<()> {
        if self.state == SlpState::Connected {
            if let Some(ref mut stream) = self.stream {
                // Send reset
                let reset = SlpPacket::reset();
                let _ = stream.write_all(&reset.encode());
            }
        }
        
        self.stream = None;
        self.state = SlpState::Disconnected;
        Ok(())
    }
    
    /// Receive a packet from stream
    fn receive_packet(&mut self) -> Result<SlpPacket> {
        let stream = self.stream.as_mut().ok_or(PilotError::SockDisconnected)?;
        let mut start_found = false;
        let mut buffer = Vec::new();
        let mut byte = [0u8; 1];
        
        while stream.read(&mut byte).map_err(|_| PilotError::SockIo)? > 0 {
            if byte[0] == 0xC0 {
                if start_found {
                    // End marker - decode packet
                    buffer.push(0xC0);
                    return SlpPacket::decode(&buffer);
                } else {
                    start_found = true;
                    buffer.push(0xC0);
                }
            } else if start_found {
                buffer.push(byte[0]);
            }
            
            if buffer.len() > 65536 {
                return Err(PilotError::ProtBadPacket);
            }
        }
        
        Err(PilotError::SockDisconnected)
    }
    
    /// Send data with reliable delivery
    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        loop {
            let stream = self.stream.as_mut()
                .ok_or(PilotError::SockDisconnected)?;
            
            let packet = SlpPacket::data(self.seq_send, data.to_vec());
            stream.write_all(&packet.encode())
                .map_err(|_| PilotError::SockIo)?;
            stream.flush().map_err(|_| PilotError::SockIo)?;
            let _ = stream;
            
            // Wait for ACK
            let response = self.receive_packet()?;
            
            match response.packet_type {
                SlpPacketType::Ack if response.seq_num == self.seq_send => {
                    self.seq_send = self.seq_send.wrapping_add(1);
                    return Ok(());
                }
                SlpPacketType::Nak | SlpPacketType::Nack => {
                    // Retry
                    continue;
                }
                _ => {
                    return Err(PilotError::ProtBadPacket);
                }
            }
        }
    }
    
    /// Get connection state
    pub fn state(&self) -> SlpState {
        self.state
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state == SlpState::Connected && self.stream.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slp_packet_ack() {
        let ack = SlpPacket::ack(5);
        assert!(matches!(ack.packet_type, SlpPacketType::Ack));
        assert_eq!(ack.seq_num, 5);
        assert!(ack.data.is_empty());
    }

    #[test]
    fn test_slp_packet_encode_decode() {
        let original = SlpPacket::data(3, vec![0x01, 0x02, 0x03, 0x04]);
        let encoded = original.encode();
        
        // Should start with 0xC0
        assert_eq!(encoded[0], 0xC0);
        // Should end with 0xC0
        assert_eq!(encoded[encoded.len()-1], 0xC0);
        
        // Can decode
        let decoded = SlpPacket::decode(&encoded).unwrap();
        assert!(matches!(decoded.packet_type, SlpPacketType::Data));
        assert_eq!(decoded.seq_num, 3);
        assert_eq!(decoded.data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_slip_escaping() {
        let packet = SlpPacket::data(0, vec![0xC0, 0xDB, 0x42]);
        let encoded = packet.encode();
        let decoded = SlpPacket::decode(&encoded).unwrap();
        
        assert_eq!(decoded.data, vec![0xC0, 0xDB, 0x42]);
    }

    #[test]
    fn test_checksum() {
        let packet = SlpPacket::data(1, vec![0x10, 0x20]);
        let checksum = packet.calculate_checksum();
        // Checksum is sum of header bytes + data
        let header_sum = 0x00u8.wrapping_add(0x01).wrapping_add(0x00).wrapping_add(0x02);
        let data_sum = 0x10u8.wrapping_add(0x20);
        assert_eq!(checksum, header_sum.wrapping_add(data_sum));
    }

    #[test]
    fn test_checksum_escaping_roundtrip() {
        // checksum = 0x00 + 0x00 + 0x00 + 0x01 + 0xBF = 0xC0 (SLIP END marker)
        let packet = SlpPacket::data(0, vec![0xBF]);
        let encoded = packet.encode();
        let decoded = SlpPacket::decode(&encoded).unwrap();
        assert_eq!(decoded.data, vec![0xBF]);
        assert_eq!(decoded.checksum, 0xC0);

        // checksum = 0x00 + 0x00 + 0x00 + 0x01 + 0xDA = 0xDB (SLIP ESC marker)
        let packet2 = SlpPacket::data(0, vec![0xDA]);
        let encoded2 = packet2.encode();
        let decoded2 = SlpPacket::decode(&encoded2).unwrap();
        assert_eq!(decoded2.data, vec![0xDA]);
        assert_eq!(decoded2.checksum, 0xDB);
    }
}
