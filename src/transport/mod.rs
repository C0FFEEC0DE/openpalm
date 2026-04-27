//! Transport layer for Palm device communication
//!
//! This module provides transport abstractions for serial, USB, and Bluetooth connections.

mod serial;
pub mod usb;

#[cfg(feature = "serial")]
pub use serial::{Serial, SerialParams};
pub use usb::{Usb, UsbParams};

use std::io::{Read, Write};

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// Connection trait for transport abstraction
pub trait Connection: Read + Write + Send {
    /// Connect to device
    fn connect(&mut self) -> crate::error::Result<()>;
    
    /// Disconnect from device
    fn disconnect(&mut self) -> crate::error::Result<()>;
    
    /// Check if connected
    fn is_connected(&self) -> bool;
    
    /// Get connection state
    fn state(&self) -> ConnectionState {
        if self.is_connected() {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        }
    }
    
    /// Set timeout for operations
    fn set_timeout(&mut self, timeout: std::time::Duration) {
        let _ = timeout;
    }
}

impl<T: Connection + ?Sized> Connection for Box<T> {
    fn connect(&mut self) -> crate::error::Result<()> {
        (**self).connect()
    }
    
    fn disconnect(&mut self) -> crate::error::Result<()> {
        (**self).disconnect()
    }
    
    fn is_connected(&self) -> bool {
        (**self).is_connected()
    }
    
    fn state(&self) -> ConnectionState {
        (**self).state()
    }
}

/// Mock connection for testing
pub struct MockConnection {
    connected: bool,
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
    read_pos: usize,
}

impl MockConnection {
    /// Create a new mock connection
    pub fn new() -> Self {
        Self {
            connected: false,
            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
            read_pos: 0,
        }
    }
    
    /// Create with initial read data
    pub fn with_data(data: Vec<u8>) -> Self {
        Self {
            connected: false,
            read_buffer: data,
            write_buffer: Vec::new(),
            read_pos: 0,
        }
    }
    
    /// Get data written to this connection
    pub fn written_data(&self) -> &[u8] {
        &self.write_buffer
    }
    
    /// Set data to be read
    pub fn set_read_data(&mut self, data: Vec<u8>) {
        self.read_buffer = data;
        self.read_pos = 0;
    }
    
    /// Clear write buffer
    pub fn clear_write_buffer(&mut self) {
        self.write_buffer.clear();
    }
}

impl Default for MockConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl Connection for MockConnection {
    fn connect(&mut self) -> crate::error::Result<()> {
        self.connected = true;
        Ok(())
    }
    
    fn disconnect(&mut self) -> crate::error::Result<()> {
        self.connected = false;
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Read for MockConnection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if !self.connected {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "mock connection not connected"
            ));
        }
        
        if self.read_pos >= self.read_buffer.len() {
            return Ok(0);
        }
        
        let len = std::cmp::min(buf.len(), self.read_buffer.len() - self.read_pos);
        buf[..len].copy_from_slice(&self.read_buffer[self.read_pos..self.read_pos + len]);
        self.read_pos += len;
        Ok(len)
    }
}

impl Write for MockConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if !self.connected {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "mock connection not connected"
            ));
        }
        
        self.write_buffer.extend_from_slice(buf);
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::fmt::Debug for MockConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockConnection")
            .field("connected", &self.connected)
            .field("read_buffer_len", &self.read_buffer.len())
            .field("write_buffer_len", &self.write_buffer.len())
            .finish()
    }
}
