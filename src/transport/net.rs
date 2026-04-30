//! TCP/IP network transport for Palm devices
//!
//! Provides network HotSync support using std::net::TcpStream.
//! Palm devices typically listen on port 14237 for network HotSync.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::error::{PilotError, Result};
use crate::transport::{Connection, ConnectionState};

/// Network connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InetState {
    /// Not connected or bound
    Disconnected,
    /// Bound to a local address
    Bound,
    /// Listening for incoming connections
    Listening,
    /// Connected to a remote peer
    Connected,
    /// In the process of connecting
    Connecting,
}

/// Network connection parameters
#[derive(Debug, Clone)]
pub struct NetParams {
    /// Hostname or IP address of the Palm device
    pub host: String,
    /// TCP port (default: 14237 for HotSync)
    pub port: u16,
    /// Connection timeout
    pub timeout: Duration,
}

impl Default for NetParams {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 14237,
            timeout: Duration::from_secs(30),
        }
    }
}

impl NetParams {
    /// Create new network params for the given host
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            ..Default::default()
        }
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// TCP/IP network connection for Palm device HotSync
///
/// Wraps a `std::net::TcpStream` and implements `Read` + `Write`
/// by delegating to the underlying TCP stream.
pub struct InetConnection {
    /// Connection parameters
    params: NetParams,
    /// Current connection state
    state: InetState,
    /// Underlying TCP stream (None when not connected)
    stream: Option<TcpStream>,
    /// TCP listener for server mode
    listener: Option<TcpListener>,
    /// Bytes received
    rx_bytes: u64,
    /// Bytes transmitted
    tx_bytes: u64,
    /// Receive errors
    rx_errors: u64,
    /// Transmit errors
    tx_errors: u64,
    /// Whether the stream is in non-blocking mode
    nonblocking: bool,
}

impl InetConnection {
    /// Create a new network connection with the given parameters
    pub fn new(params: NetParams) -> Self {
        Self {
            params,
            state: InetState::Disconnected,
            stream: None,
            listener: None,
            rx_bytes: 0,
            tx_bytes: 0,
            rx_errors: 0,
            tx_errors: 0,
            nonblocking: false,
        }
    }

    /// Bind to a local address for server mode
    pub fn bind(&mut self, addr: impl ToSocketAddrs) -> Result<()> {
        let listener = TcpListener::bind(addr).map_err(|_| PilotError::SockIo)?;
        self.listener = Some(listener);
        self.state = InetState::Bound;
        Ok(())
    }

    /// Transition from Bound to Listening state
    pub fn listen(&mut self) -> Result<()> {
        if self.state != InetState::Bound {
            return Err(PilotError::SockInvalid);
        }
        self.state = InetState::Listening;
        Ok(())
    }

    /// Accept an incoming connection
    pub fn accept(&mut self) -> Result<()> {
        if self.state != InetState::Listening {
            return Err(PilotError::SockInvalid);
        }

        let listener = self.listener.as_ref().ok_or(PilotError::SockInvalid)?;
        let (stream, _addr) = listener.accept().map_err(|_| PilotError::SockIo)?;

        stream
            .set_read_timeout(Some(self.params.timeout))
            .map_err(|_| PilotError::SockIo)?;
        stream
            .set_write_timeout(Some(self.params.timeout))
            .map_err(|_| PilotError::SockIo)?;

        self.stream = Some(stream);
        self.nonblocking = false;
        self.state = InetState::Connected;
        Ok(())
    }

    /// Get the local bound address
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        self.listener
            .as_ref()
            .map(|l| l.local_addr().map_err(|_| PilotError::SockIo))
            .unwrap_or(Err(PilotError::SockInvalid))
    }

    /// Connect to the Palm device over TCP/IP
    pub fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.params.host, self.params.port);

        let addrs: Vec<_> = addr
            .to_socket_addrs()
            .map_err(|_| PilotError::SockIo)?
            .collect();

        if addrs.is_empty() {
            return Err(PilotError::SockIo);
        }

        self.state = InetState::Connecting;

        let mut stream = None;
        for a in &addrs {
            match TcpStream::connect_timeout(a, self.params.timeout) {
                Ok(s) => {
                    stream = Some(s);
                    break;
                }
                Err(_) => {}
            }
        }
        let stream = match stream {
            Some(s) => s,
            None => {
                self.state = InetState::Disconnected;
                return Err(PilotError::SockIo);
            }
        };

        stream
            .set_read_timeout(Some(self.params.timeout))
            .map_err(|_| PilotError::SockIo)?;
        stream
            .set_write_timeout(Some(self.params.timeout))
            .map_err(|_| PilotError::SockIo)?;

        self.stream = Some(stream);
        self.nonblocking = false;
        self.state = InetState::Connected;
        Ok(())
    }

    /// Disconnect from the Palm device
    pub fn disconnect(&mut self) -> Result<()> {
        // Dropping the TcpStream closes the connection
        self.stream = None;
        self.listener = None;
        self.state = InetState::Disconnected;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.state == InetState::Connected
    }

    /// Get the current connection state
    pub fn state(&self) -> InetState {
        self.state
    }

    /// Set read/write timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.params.timeout = timeout;
        if let Some(ref stream) = self.stream {
            let _ = stream.set_read_timeout(Some(timeout));
            let _ = stream.set_write_timeout(Some(timeout));
        }
    }

    /// Get the host
    pub fn host(&self) -> &str {
        &self.params.host
    }

    /// Get the port
    pub fn port(&self) -> u16 {
        self.params.port
    }

    /// Get the timeout
    pub fn timeout(&self) -> Duration {
        self.params.timeout
    }

    /// Get total bytes received
    pub fn rx_bytes(&self) -> u64 {
        self.rx_bytes
    }

    /// Get total bytes transmitted
    pub fn tx_bytes(&self) -> u64 {
        self.tx_bytes
    }

    /// Get receive error count
    pub fn rx_errors(&self) -> u64 {
        self.rx_errors
    }

    /// Get transmit error count
    pub fn tx_errors(&self) -> u64 {
        self.tx_errors
    }
}

impl Read for InetConnection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "network connection not connected",
            )
        })?;

        match stream.read(buf) {
            Ok(n) => {
                self.rx_bytes += n as u64;
                Ok(n)
            }
            Err(e) => {
                self.rx_errors += 1;
                Err(e)
            }
        }
    }
}

impl Write for InetConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "network connection not connected",
            )
        })?;

        match stream.write(buf) {
            Ok(0) => {
                self.tx_errors += 1;
                Err(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "write returned 0 bytes",
                ))
            }
            Ok(n) => {
                self.tx_bytes += n as u64;
                Ok(n)
            }
            Err(e) => {
                self.tx_errors += 1;
                Err(e)
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.stream.as_mut() {
            Some(stream) => stream.flush(),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "network connection not connected",
            )),
        }
    }
}

impl Connection for InetConnection {
    fn connect(&mut self) -> Result<()> {
        self.connect()
    }

    fn disconnect(&mut self) -> Result<()> {
        self.disconnect()
    }

    fn is_connected(&self) -> bool {
        self.is_connected()
    }

    fn state(&self) -> ConnectionState {
        if self.is_connected() {
            ConnectionState::Connected
        } else if self.state == InetState::Connecting {
            ConnectionState::Connecting
        } else {
            ConnectionState::Disconnected
        }
    }

    fn set_timeout(&mut self, timeout: Duration) {
        self.set_timeout(timeout)
    }

    fn drain_input(&mut self) -> std::io::Result<()> {
        let stream = match self.stream.as_mut() {
            Some(s) => s,
            None => return Ok(()),
        };

        let was_nonblocking = self.nonblocking;
        stream.set_nonblocking(true)?;
        self.nonblocking = true;
        let mut discard = [0u8; 256];
        loop {
            match stream.read(&mut discard) {
                Ok(0) => break,
                Ok(n) => {
                    self.rx_bytes += n as u64;
                    continue;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
        stream.set_nonblocking(was_nonblocking)?;
        self.nonblocking = was_nonblocking;
        Ok(())
    }
}

impl std::fmt::Debug for InetConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InetConnection")
            .field("host", &self.params.host)
            .field("port", &self.params.port)
            .field("timeout", &self.params.timeout)
            .field("state", &self.state)
            .field("connected", &self.is_connected())
            .field("rx_bytes", &self.rx_bytes)
            .field("tx_bytes", &self.tx_bytes)
            .field("rx_errors", &self.rx_errors)
            .field("tx_errors", &self.tx_errors)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net_params_default() {
        let params = NetParams::default();
        assert_eq!(params.host, "");
        assert_eq!(params.port, 14237);
        assert_eq!(params.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_net_params_new() {
        let params = NetParams::new("192.168.1.100")
            .with_port(4242)
            .with_timeout(Duration::from_secs(10));
        assert_eq!(params.host, "192.168.1.100");
        assert_eq!(params.port, 4242);
        assert_eq!(params.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_net_connection_initial_state() {
        let params = NetParams::new("192.168.1.100");
        let conn = InetConnection::new(params);
        assert!(!conn.is_connected());
        assert_eq!(conn.host(), "192.168.1.100");
        assert_eq!(conn.port(), 14237);
        assert_eq!(conn.state(), InetState::Disconnected);
        assert_eq!(conn.rx_bytes(), 0);
        assert_eq!(conn.tx_bytes(), 0);
        assert_eq!(conn.rx_errors(), 0);
        assert_eq!(conn.tx_errors(), 0);
    }

    #[test]
    fn test_net_connection_disconnect_when_not_connected() {
        let params = NetParams::new("192.168.1.100");
        let mut conn = InetConnection::new(params);
        // Should be a no-op, not panic
        assert!(conn.disconnect().is_ok());
        assert_eq!(conn.state(), InetState::Disconnected);
    }

    #[test]
    fn test_net_connection_read_when_not_connected() {
        let params = NetParams::new("192.168.1.100");
        let mut conn = InetConnection::new(params);
        let mut buf = [0u8; 16];
        let result = conn.read(&mut buf);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotConnected);
    }

    #[test]
    fn test_net_connection_write_when_not_connected() {
        let params = NetParams::new("192.168.1.100");
        let mut conn = InetConnection::new(params);
        let result = conn.write(b"hello");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotConnected);
    }

    #[test]
    fn test_net_connection_set_timeout() {
        let params = NetParams::new("192.168.1.100");
        let mut conn = InetConnection::new(params);
        let new_timeout = Duration::from_secs(5);
        conn.set_timeout(new_timeout);
        assert_eq!(conn.timeout(), new_timeout);
    }

    #[test]
    fn test_net_connection_debug() {
        let params = NetParams::new("192.168.1.100");
        let conn = InetConnection::new(params);
        let debug_str = format!("{:?}", conn);
        assert!(debug_str.contains("192.168.1.100"));
        assert!(debug_str.contains("14237"));
    }
}
