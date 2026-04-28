//! Transport layer for Palm device communication
//!
//! This module provides transport abstractions for serial, USB, and Bluetooth connections.

#[cfg(feature = "serial")]
pub mod serial;
#[cfg(feature = "usb")]
pub mod usb;

#[cfg(feature = "serial")]
pub use serial::Serial;
#[cfg(feature = "usb")]
pub use usb::Usb;

use std::io::{Read, Write};
use async_trait::async_trait;

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

/// Async connection trait for async/await operations
/// 
/// This trait provides async versions of the Connection operations.
/// Implement this trait for async transport layers.
#[async_trait]
pub trait AsyncConnection: Send + Sync {
    /// Connect to device asynchronously
    async fn connect_async(&mut self) -> crate::error::Result<()>;
    
    /// Disconnect from device asynchronously
    async fn disconnect_async(&mut self) -> crate::error::Result<()>;
    
    /// Check if connected
    fn is_connected(&self) -> bool;
    
    /// Read data asynchronously
    async fn read_async(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    
    /// Write data asynchronously  
    async fn write_async(&mut self, buf: &[u8]) -> std::io::Result<usize>;
    
    /// Flush write buffer
    async fn flush_async(&mut self) -> std::io::Result<()>;
    
    /// Get connection state
    fn state(&self) -> ConnectionState {
        if self.is_connected() {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        }
    }
}

/// Adapter to convert sync Connection to AsyncConnection
///
/// This allows sync connections to be used with async code.
/// Uses interior mutability for thread safety.
pub struct AsyncConnectionAdapter<T> {
    inner: std::sync::Mutex<T>,
}

impl<T> AsyncConnectionAdapter<T> {
    /// Create a new async adapter from a sync connection
    pub fn new(inner: T) -> Self {
        Self {
            inner: std::sync::Mutex::new(inner),
        }
    }

    /// Consume adapter and return inner connection
    pub fn into_inner(self) -> T {
        self.inner.into_inner().unwrap()
    }
}

#[async_trait]
impl<T: Connection + Send + 'static> AsyncConnection for AsyncConnectionAdapter<T> {
    async fn connect_async(&mut self) -> crate::error::Result<()> {
        let mut guard = self.inner.lock().unwrap();
        guard.connect()
    }

    async fn disconnect_async(&mut self) -> crate::error::Result<()> {
        let mut guard = self.inner.lock().unwrap();
        guard.disconnect()
    }

    fn is_connected(&self) -> bool {
        if let Ok(guard) = self.inner.lock() {
            guard.is_connected()
        } else {
            false
        }
    }

    async fn read_async(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut guard = self.inner.lock().unwrap();
        guard.read(buf)
    }

    async fn write_async(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut guard = self.inner.lock().unwrap();
        guard.write(buf)
    }

    async fn flush_async(&mut self) -> std::io::Result<()> {
        let mut guard = self.inner.lock().unwrap();
        guard.flush()
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
#[derive(Clone)]
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
                "mock connection not connected",
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
                "mock connection not connected",
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
