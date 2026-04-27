//! PilotSocket - Socket-like interface for Palm device communication
//!
//! This module provides the pi_socket_t abstraction from pilot-link.

use crate::error::{PilotError, Result};
use crate::transport::{Connection, ConnectionState, MockConnection};
#[cfg(feature = "serial")]
use crate::transport::Serial;
#[cfg(feature = "usb")]
use crate::transport::Usb;
use crate::protocol::dlp::{DlpClient, ProtocolVersion};
use std::io::{Read, Write};

/// Socket state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Closed,
    Listening,
    Connected,
    Connecting,
}

/// Socket type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Stream,
    Datagram,
    Raw,
}

/// Protocol family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolFamily {
    /// Serial connection
    Serial,
    /// USB connection
    Usb,
    /// Network (TCP/IP)
    Net,
    /// Bluetooth
    Bluetooth,
}

/// Socket options
#[derive(Debug, Clone)]
pub struct SocketOptions {
    /// Timeout for operations
    pub timeout_ms: u32,
    /// Enable keep-alive
    pub keep_alive: bool,
    /// Buffer sizes
    pub send_buffer_size: usize,
    pub recv_buffer_size: usize,
    /// SLIP mode
    pub slip_mode: bool,
    /// Checksum mode
    pub checksum_mode: bool,
}

impl Default for SocketOptions {
    fn default() -> Self {
        Self {
            timeout_ms: 30000,
            keep_alive: true,
            send_buffer_size: 65536,
            recv_buffer_size: 65536,
            slip_mode: true,
            checksum_mode: true,
        }
    }
}

/// Inner transport connection (re-exported for DLP client)
#[derive(Debug, Clone)]
pub enum TransportConnection {
    #[cfg(feature = "serial")]
    Serial(Serial),
    #[cfg(feature = "usb")]
    Usb(Usb),
    Mock(MockConnection),
}

impl TransportConnection {
    fn is_connected(&self) -> bool {
        match self {
            #[cfg(feature = "serial")]
            TransportConnection::Serial(s) => s.is_connected(),
            #[cfg(feature = "usb")]
            TransportConnection::Usb(u) => u.is_connected(),
            TransportConnection::Mock(m) => m.is_connected(),
        }
    }
    
    fn connect(&mut self) -> Result<()> {
        match self {
            #[cfg(feature = "serial")]
            TransportConnection::Serial(s) => s.connect(),
            #[cfg(feature = "usb")]
            TransportConnection::Usb(u) => u.connect(),
            TransportConnection::Mock(m) => m.connect(),
        }
    }
    
    fn disconnect(&mut self) -> Result<()> {
        match self {
            #[cfg(feature = "serial")]
            TransportConnection::Serial(s) => s.disconnect(),
            #[cfg(feature = "usb")]
            TransportConnection::Usb(u) => u.disconnect(),
            TransportConnection::Mock(m) => m.disconnect(),
        }
    }
}

impl Read for TransportConnection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "serial")]
            TransportConnection::Serial(s) => s.read(buf),
            #[cfg(feature = "usb")]
            TransportConnection::Usb(u) => u.read(buf),
            TransportConnection::Mock(m) => m.read(buf),
        }
    }
}

impl Write for TransportConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            #[cfg(feature = "serial")]
            TransportConnection::Serial(s) => s.write(buf),
            #[cfg(feature = "usb")]
            TransportConnection::Usb(u) => u.write(buf),
            TransportConnection::Mock(m) => m.write(buf),
        }
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            #[cfg(feature = "serial")]
            TransportConnection::Serial(s) => s.flush(),
            #[cfg(feature = "usb")]
            TransportConnection::Usb(u) => u.flush(),
            TransportConnection::Mock(m) => m.flush(),
        }
    }
}

/// A socket for communicating with a Palm device
pub struct PilotSocket {
    /// Socket state
    state: SocketState,
    /// Socket type
    socket_type: SocketType,
    /// Protocol family
    protocol_family: ProtocolFamily,
    /// Protocol version
    protocol_version: ProtocolVersion,
    /// DLP version negotiated
    dlp_version: Option<ProtocolVersion>,
    /// Maximum record size supported
    max_record_size: u32,
    /// Socket options
    options: SocketOptions,
    /// Transport connection
    transport: Option<TransportConnection>,
    /// DLP client
    dlp_client: Option<DlpClient>,
    /// Socket descriptor
    sd: i32,
}

impl PilotSocket {
    /// Create a new socket
    pub fn new(family: ProtocolFamily, socket_type: SocketType) -> Self {
        static mut SOCKET_COUNTER: i32 = 0;
        
        let sd = unsafe {
            SOCKET_COUNTER += 1;
            SOCKET_COUNTER
        };
        
        Self {
            state: SocketState::Closed,
            socket_type,
            protocol_family: family,
            protocol_version: ProtocolVersion::default(),
            dlp_version: None,
            max_record_size: 0xFFFF,
            options: SocketOptions::default(),
            transport: None,
            dlp_client: None,
            sd,
        }
    }
    
    /// Create a stream socket for serial
    pub fn serial(_port: &str) -> Self {
        let mut socket = Self::new(ProtocolFamily::Serial, SocketType::Stream);
        #[cfg(feature = "serial")]
        {
            socket.transport = Some(TransportConnection::Serial(Serial::from_port(port)));
        }
        socket
    }
    
    /// Create a stream socket for USB
    pub fn usb() -> Self {
        let mut socket = Self::new(ProtocolFamily::Usb, SocketType::Stream);
        #[cfg(feature = "usb")]
        {
            socket.transport = Some(TransportConnection::Usb(Usb::new_palm()));
        }
        socket
    }
    
    /// Create a mock socket for testing
    pub fn mock() -> Self {
        let mut socket = Self::new(ProtocolFamily::Serial, SocketType::Stream);
        socket.transport = Some(TransportConnection::Mock(MockConnection::new()));
        socket
    }
    
    // ========================================================================
    // Connection Management
    // ========================================================================
    
    /// Connect to a device
    pub fn connect(&mut self) -> Result<()> {
        if self.state != SocketState::Closed {
            return Err(PilotError::SockInvalid);
        }
        
        let transport = self.transport.as_mut()
            .ok_or(PilotError::SockInvalid)?;
        
        transport.connect()?;
        self.state = SocketState::Connected;
        
        // Create DLP client with the transport
        if let Some(ref transport) = self.transport {
            self.dlp_client = Some(DlpClient::new(transport.clone()));
        }
        
        Ok(())
    }
    
    /// Disconnect from device
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(ref mut transport) = self.transport {
            transport.disconnect()?;
        }
        
        self.dlp_client = None;
        self.state = SocketState::Closed;
        
        Ok(())
    }
    
    /// Check if socket is connected
    pub fn is_connected(&self) -> bool {
        self.state == SocketState::Connected && 
            self.transport.as_ref().map_or(false, |t| t.is_connected())
    }
    
    /// Get connection state
    pub fn connection_state(&self) -> ConnectionState {
        match self.state {
            SocketState::Connected => ConnectionState::Connected,
            SocketState::Connecting => ConnectionState::Connecting,
            SocketState::Closed | SocketState::Listening => ConnectionState::Disconnected,
        }
    }
    
    // ========================================================================
    // Transport Access
    // ========================================================================
    
    /// Get mutable transport reference for protocol layer
    pub fn transport_mut(&mut self) -> Option<&mut TransportConnection> {
        self.transport.as_mut()
    }
    
    /// Get transport reference
    pub fn transport(&self) -> Option<&TransportConnection> {
        self.transport.as_ref()
    }
    
    // ========================================================================
    // DLP Operations
    // ========================================================================
    
    /// Get DLP client reference
    pub fn dlp(&self) -> Option<&DlpClient> {
        self.dlp_client.as_ref()
    }
    
    /// Get mutable DLP client reference
    pub fn dlp_mut(&mut self) -> Option<&mut DlpClient> {
        self.dlp_client.as_mut()
    }
    
    /// Read system info
    pub async fn read_sys_info(&mut self) -> Result<crate::protocol::dlp::SystemInfo> {
        if let Some(ref mut client) = self.dlp_client {
            client.read_sys_info().await
        } else {
            Err(PilotError::DlpSocket)
        }
    }
    
    /// Read user info
    pub async fn read_user_info(&mut self) -> Result<crate::protocol::dlp::UserInfo> {
        if let Some(ref mut client) = self.dlp_client {
            client.read_user_info().await
        } else {
            Err(PilotError::DlpSocket)
        }
    }
    
    /// List databases
    pub async fn list_databases(&mut self) -> Result<Vec<crate::database::DatabaseInfo>> {
        if let Some(ref mut client) = self.dlp_client {
            client.read_db_list(0, crate::protocol::dlp::DlpDBListFlag::Ram, 0).await
        } else {
            Err(PilotError::DlpSocket)
        }
    }
    
    /// Open database
    pub async fn open_database(&mut self, name: &str, mode: crate::protocol::dlp::DlpOpenMode) 
        -> Result<crate::database::DatabaseHandle> 
    {
        if let Some(ref mut client) = self.dlp_client {
            client.open_db(0, name, mode).await
        } else {
            Err(PilotError::DlpSocket)
        }
    }
    
    /// Close database
    pub async fn close_database(&mut self, handle: crate::database::DatabaseHandle) -> Result<()> {
        if let Some(ref mut client) = self.dlp_client {
            client.close_db(handle).await
        } else {
            Err(PilotError::DlpSocket)
        }
    }
    
    /// Read record by index
    pub async fn read_record(&mut self, handle: crate::database::DatabaseHandle, index: u32) 
        -> Result<crate::database::Record> 
    {
        if let Some(ref mut client) = self.dlp_client {
            client.read_record(handle, index).await
        } else {
            Err(PilotError::DlpSocket)
        }
    }
    
    /// Read next modified record
    pub async fn read_next_modified(&mut self, handle: crate::database::DatabaseHandle) 
        -> Result<Option<crate::database::Record>> 
    {
        if let Some(ref mut client) = self.dlp_client {
            client.read_next_modified_rec(handle).await
        } else {
            Err(PilotError::DlpSocket)
        }
    }
    
    // ========================================================================
    // Properties
    // ========================================================================
    
    /// Get socket descriptor
    pub fn sd(&self) -> i32 {
        self.sd
    }
    
    /// Get socket state
    pub fn state(&self) -> SocketState {
        self.state
    }
    
    /// Get socket type
    pub fn socket_type(&self) -> SocketType {
        self.socket_type
    }
    
    /// Get protocol family
    pub fn protocol_family(&self) -> ProtocolFamily {
        self.protocol_family
    }
    
    /// Get protocol version
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.protocol_version
    }
    
    /// Set protocol version
    pub fn set_protocol_version(&mut self, version: ProtocolVersion) {
        self.protocol_version = version;
    }
    
    /// Get DLP version
    pub fn dlp_version(&self) -> Option<ProtocolVersion> {
        self.dlp_version
    }
    
    /// Get maximum record size
    pub fn max_record_size(&self) -> u32 {
        self.max_record_size
    }
    
    /// Get options
    pub fn options(&self) -> &SocketOptions {
        &self.options
    }
    
    /// Set options
    pub fn set_options(&mut self, options: SocketOptions) {
        self.options = options;
    }
}

impl Default for PilotSocket {
    fn default() -> Self {
        Self::new(ProtocolFamily::Usb, SocketType::Stream)
    }
}

impl std::fmt::Debug for PilotSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PilotSocket")
            .field("sd", &self.sd)
            .field("state", &self.state)
            .field("socket_type", &self.socket_type)
            .field("protocol_family", &self.protocol_family)
            .field("dlp_version", &self.dlp_version)
            .field("max_record_size", &self.max_record_size)
            .finish()
    }
}
