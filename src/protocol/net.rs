//! NET protocol for Palm OS
//!
//! This module implements the NET protocol for network communication over
//! the Palm serial/USB connection.

use crate::error::{PilotError, Result};

/// NET protocol constants
pub mod constants {
    /// NET protocol version
    pub const NET_VERSION: u8 = 1;
    
    /// Maximum packet size
    pub const NET_MAX_PACKET: usize = 65535;
    
    /// Default timeout (ms)
    pub const NET_TIMEOUT: u32 = 5000;
    
    /// Maximum retries
    pub const NET_MAX_RETRIES: u8 = 3;
    
    /// Protocol types
    pub const NET_PROTO_TCP: u8 = 0x06;
    pub const NET_PROTO_UDP: u8 = 0x11;
    pub const NET_PROTO_ICMP: u8 = 0x01;
    
    /// Port numbers
    pub const PORT_HANDSYNC: u16 = 14237;
    pub const PORT_DEBUG: u16 = 14238;
    pub const PORT_CRASH: u16 = 14239;
}

/// NET command types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NetCommand {
    /// Open connection
    Open = 0x01,
    /// Close connection
    Close = 0x02,
    /// Send data
    Send = 0x03,
    /// Receive data
    Receive = 0x04,
    /// Get status
    Status = 0x05,
    /// Reset
    Reset = 0x06,
    /// Open response
    OpenResp = 0x81,
    /// Close response
    CloseResp = 0x82,
    /// Send response
    SendResp = 0x83,
    /// Receive response
    ReceiveResp = 0x84,
    /// Status response
    StatusResp = 0x85,
    /// Error response
    Error = 0xFF,
}

impl NetCommand {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x01 => Some(NetCommand::Open),
            0x02 => Some(NetCommand::Close),
            0x03 => Some(NetCommand::Send),
            0x04 => Some(NetCommand::Receive),
            0x05 => Some(NetCommand::Status),
            0x06 => Some(NetCommand::Reset),
            0x81 => Some(NetCommand::OpenResp),
            0x82 => Some(NetCommand::CloseResp),
            0x83 => Some(NetCommand::SendResp),
            0x84 => Some(NetCommand::ReceiveResp),
            0x85 => Some(NetCommand::StatusResp),
            0xFF => Some(NetCommand::Error),
            _ => None,
        }
    }
}

/// NET packet header
#[derive(Debug, Clone)]
pub struct NetPacket {
    /// Command
    pub command: NetCommand,
    /// Connection ID
    pub conn_id: u8,
    /// Sequence number
    pub seq: u16,
    /// Data length
    pub length: u16,
    /// Data
    pub data: Vec<u8>,
}

impl NetPacket {
    /// Create new packet
    pub fn new(command: NetCommand, conn_id: u8, seq: u16) -> Self {
        Self {
            command,
            conn_id,
            seq,
            length: 0,
            data: Vec::new(),
        }
    }

    /// Pack to bytes
    pub fn pack(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Header: command (1) + conn_id (1) + seq (2) + length (2)
        bytes.push(self.command as u8);
        bytes.push(self.conn_id);
        bytes.extend_from_slice(&self.seq.to_be_bytes());
        bytes.extend_from_slice(&((self.data.len() as u16).to_be_bytes()));
        
        // Data
        bytes.extend_from_slice(&self.data);
        
        bytes
    }

    /// Parse from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 6 {
            return Err(PilotError::InvalidData("NET packet too short".into()));
        }

        let command = NetCommand::from_u8(data[0])
            .ok_or_else(|| PilotError::InvalidData("Invalid NET command".into()))?;
        
        let conn_id = data[1];
        let seq = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);

        if (data.len() - 6) as u16 != length {
            return Err(PilotError::InvalidData("NET packet length mismatch".into()));
        }

        let packet_data = data[6..].to_vec();

        Ok(Self {
            command,
            conn_id,
            seq,
            length,
            data: packet_data,
        })
    }

    /// Create open packet
    pub fn open(conn_id: u8, seq: u16, protocol: u8, local_port: u16, remote_port: u16) -> Self {
        let mut data = Vec::new();
        data.push(protocol);
        data.extend_from_slice(&local_port.to_be_bytes());
        data.extend_from_slice(&remote_port.to_be_bytes());
        
        let mut packet = Self::new(NetCommand::Open, conn_id, seq);
        packet.data = data;
        packet.length = packet.data.len() as u16;
        packet
    }

    /// Create close packet
    pub fn close(conn_id: u8, seq: u16) -> Self {
        Self::new(NetCommand::Close, conn_id, seq)
    }

    /// Create send packet
    pub fn send(conn_id: u8, seq: u16, data: Vec<u8>) -> Self {
        let mut packet = Self::new(NetCommand::Send, conn_id, seq);
        packet.data = data;
        packet.length = packet.data.len() as u16;
        packet
    }

    /// Create receive packet
    pub fn receive(conn_id: u8, seq: u16, max_length: u16) -> Self {
        let mut packet = Self::new(NetCommand::Receive, conn_id, seq);
        packet.data = max_length.to_be_bytes().to_vec();
        packet.length = 2;
        packet
    }
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetState {
    Closed,
    Opening,
    Open,
    Closing,
    Error,
}

/// NET connection
#[derive(Debug, Clone)]
pub struct NetConnection {
    /// Connection ID
    pub id: u8,
    /// State
    pub state: NetState,
    /// Protocol
    pub protocol: u8,
    /// Local port
    pub local_port: u16,
    /// Remote port
    pub remote_port: u16,
    /// Sequence number
    pub seq: u16,
    /// Bytes sent
    pub bytes_sent: u32,
    /// Bytes received
    pub bytes_received: u32,
}

impl NetConnection {
    /// Create new connection
    pub fn new(id: u8, protocol: u8, local_port: u16, remote_port: u16) -> Self {
        Self {
            id,
            state: NetState::Closed,
            protocol,
            local_port,
            remote_port,
            seq: 0,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }

    /// Increment sequence
    pub fn next_seq(&mut self) -> u16 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }
}

/// NET error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum NetError {
    None = 0,
    Busy = 1,
    NoMem = 2,
    BadArg = 3,
    NoConn = 4,
    Timeout = 5,
    Reset = 6,
    Refused = 7,
    NoRoute = 8,
    Unknown = 0xFFFF,
}

impl NetError {
    pub fn from_u16(val: u16) -> Self {
        match val {
            0 => NetError::None,
            1 => NetError::Busy,
            2 => NetError::NoMem,
            3 => NetError::BadArg,
            4 => NetError::NoConn,
            5 => NetError::Timeout,
            6 => NetError::Reset,
            7 => NetError::Refused,
            8 => NetError::NoRoute,
            _ => NetError::Unknown,
        }
    }
}

/// NET protocol handler
pub struct NetHandler {
    /// Connections
    connections: Vec<NetConnection>,
    /// Next connection ID
    next_id: u8,
}

impl NetHandler {
    /// Create new handler
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            next_id: 1,
        }
    }

    /// Create connection
    pub fn create_connection(&mut self, protocol: u8, local_port: u16, remote_port: u16) -> &mut NetConnection {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        
        let conn = NetConnection::new(id, protocol, local_port, remote_port);
        self.connections.push(conn);
        
        self.connections.last_mut().unwrap()
    }

    /// Get connection by ID
    pub fn get_connection(&self, id: u8) -> Option<&NetConnection> {
        self.connections.iter().find(|c| c.id == id)
    }

    /// Get connection by ID (mutable)
    pub fn get_connection_mut(&mut self, id: u8) -> Option<&mut NetConnection> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    /// Close connection
    pub fn close_connection(&mut self, id: u8) {
        self.connections.retain(|c| c.id != id);
    }

    /// Handle packet
    pub fn handle_packet(&mut self, packet: &NetPacket) -> Option<NetPacket> {
        match packet.command {
            NetCommand::Open => {
                // Parse open request
                if packet.data.len() >= 5 {
                    let protocol = packet.data[0];
                    let local_port = u16::from_be_bytes([packet.data[1], packet.data[2]]);
                    let remote_port = u16::from_be_bytes([packet.data[3], packet.data[4]]);
                    
                    let conn = self.create_connection(protocol, local_port, remote_port);
                    conn.state = NetState::Open;
                    
                    // Return response
                    let mut resp = NetPacket::new(NetCommand::OpenResp, conn.id, packet.seq);
                    resp.data = vec![0x00]; // Success
                    return Some(resp);
                }
                None
            }
            NetCommand::Close => {
                self.close_connection(packet.conn_id);
                let mut resp = NetPacket::new(NetCommand::CloseResp, packet.conn_id, packet.seq);
                resp.data = vec![0x00];
                Some(resp)
            }
            NetCommand::Send => {
                if let Some(conn) = self.get_connection_mut(packet.conn_id) {
                    conn.bytes_sent += packet.length as u32;
                    conn.next_seq();
                    
                    let mut resp = NetPacket::new(NetCommand::SendResp, conn.id, packet.seq);
                    resp.data = vec![0x00];
                    resp.length = 1;
                    return Some(resp);
                }
                None
            }
            NetCommand::Receive => {
                if let Some(conn) = self.get_connection_mut(packet.conn_id) {
                    conn.next_seq();
                    
                    let mut resp = NetPacket::new(NetCommand::ReceiveResp, conn.id, packet.seq);
                    resp.data = Vec::new(); // No data for now
                    return Some(resp);
                }
                None
            }
            NetCommand::Status => {
                let mut resp = NetPacket::new(NetCommand::StatusResp, packet.conn_id, packet.seq);
                if let Some(conn) = self.get_connection(packet.conn_id) {
                    resp.data = vec![
                        conn.state as u8,
                        0, 0, 0, // padding
                    ];
                    resp.data.extend_from_slice(&conn.bytes_sent.to_be_bytes());
                    resp.data.extend_from_slice(&conn.bytes_received.to_be_bytes());
                }
                Some(resp)
            }
            _ => None,
        }
    }
}

impl Default for NetHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_packet_pack_parse() {
        let packet = NetPacket::open(1, 100, constants::NET_PROTO_TCP, 1234, 80);
        let bytes = packet.pack();
        let parsed = NetPacket::parse(&bytes).unwrap();
        
        assert_eq!(parsed.command, NetCommand::Open);
        assert_eq!(parsed.conn_id, 1);
        assert_eq!(parsed.seq, 100);
    }

    #[test]
    fn test_net_connection() {
        let mut conn = NetConnection::new(1, constants::NET_PROTO_TCP, 1234, 80);
        assert_eq!(conn.state, NetState::Closed);
        
        conn.state = NetState::Open;
        assert_eq!(conn.next_seq(), 1);
        assert_eq!(conn.next_seq(), 2);
    }

    #[test]
    fn test_net_handler() {
        let mut handler = NetHandler::new();
        
        let conn = handler.create_connection(constants::NET_PROTO_TCP, 1234, 80);
        assert_eq!(conn.id, 1);
        
        handler.close_connection(1);
        assert!(handler.get_connection(1).is_none());
    }
}
