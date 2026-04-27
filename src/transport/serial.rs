//! Serial port transport for Palm devices
//!
//! Uses the serialport crate for cross-platform serial communication.

#[cfg(feature = "serial")]
use serialport::{SerialPort, SerialPortType, FlowControl};
use std::io::{Read, Write};
use std::time::Duration;

use crate::error::{PilotError, Result};

/// Serial connection parameters
#[derive(Debug, Clone)]
pub struct SerialParams {
    /// Port name (e.g., "/dev/ttyUSB0", "COM1")
    pub port: String,
    /// Baud rate
    pub baud_rate: u32,
    /// Data bits (5, 6, 7, or 8)
    pub data_bits: u8,
    /// Flow control
    pub flow_control: bool,
    /// XON/XOFF flow control
    pub xon_xoff: bool,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for SerialParams {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 9600,
            data_bits: 8,
            flow_control: false,
            xon_xoff: false,
            timeout_ms: 30000,
        }
    }
}

/// Serial connection implementation
pub struct Serial {
    /// Serial port name
    port_name: String,
    /// Current baud rate
    baud_rate: u32,
    /// Timeout
    timeout: Duration,
    /// Serial port handle (None when disconnected)
    #[cfg(feature = "serial")]
    port: Option<Box<dyn SerialPort>>,
}

impl Serial {
    /// Create a new serial connection with parameters
    pub fn new(params: SerialParams) -> Self {
        Self {
            port_name: params.port,
            baud_rate: params.baud_rate,
            timeout: Duration::from_millis(params.timeout_ms),
            #[cfg(feature = "serial")]
            port: None,
        }
    }
    
    /// Create from a port name (uses default parameters)
    pub fn from_port(port: &str) -> Self {
        Self::new(SerialParams {
            port: port.to_string(),
            ..Default::default()
        })
    }
    
    /// Open the serial port
    pub fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "serial")]
        {
            use serialport::{DataBits, FlowControl, Parity, StopBits};
            
            let port = serialport::new(&self.port_name, self.baud_rate)
                .data_bits(DataBits::Eight)
                .parity(Parity::None)
                .stop_bits(StopBits::One)
                .timeout(self.timeout.clone())
                .flow_control(if self.flow_control {
                    FlowControl::Hardware
                } else {
                    FlowControl::None
                })
                .open()
                .map_err(|e| PilotError::SockIo)?;
            
            self.port = Some(port);
            Ok(())
        }
        
        #[cfg(not(feature = "serial"))]
        {
            Err(PilotError::InvalidArgument)
        }
    }
    
    /// Close the serial port
    pub fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "serial")]
        {
            self.port = None;
            Ok(())
        }
        
        #[cfg(not(feature = "serial"))]
        {
            Err(PilotError::InvalidArgument)
        }
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        #[cfg(feature = "serial")]
        {
            self.port.is_some()
        }
        
        #[cfg(not(feature = "serial"))]
        {
            false
        }
    }
    
    /// Set the baud rate
    pub fn set_baud_rate(&mut self, baud: u32) -> Result<()> {
        self.baud_rate = baud;
        
        #[cfg(feature = "serial")]
        if let Some(ref mut port) = self.port {
            port.set_baud_rate(baud)
                .map_err(|_| PilotError::SockIo)?;
        }
        
        Ok(())
    }
    
    /// Get the current baud rate
    pub fn baud_rate(&self) -> u32 {
        self.baud_rate
    }
    
    /// Set timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
        
        #[cfg(feature = "serial")]
        if let Some(ref mut port) = self.port {
            let _ = port.set_timeout(timeout);
        }
    }
    
    /// Get available ports
    #[cfg(feature = "serial")]
    pub fn available_ports() -> std::io::Result<Vec<String>> {
        serialport::available_ports()
            .map(|ports| {
                ports.into_iter()
                    .map(|p| {
                        match p.port_type {
                            SerialPortType::UsbPort(_) => {
                                format!("{} (USB)", p.port_name)
                            }
                            SerialPortType::BluetoothPort => {
                                format!("{} (Bluetooth)", p.port_name)
                            }
                            _ => p.port_name
                        }
                    })
                    .collect()
            })
    }
}

impl Read for Serial {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        #[cfg(feature = "serial")]
        {
            if let Some(ref mut port) = self.port {
                port.read(buf)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "serial port not connected"
                ))
            }
        }
        
        #[cfg(not(feature = "serial"))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "serial support not enabled"
            ))
        }
    }
}

impl Write for Serial {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        #[cfg(feature = "serial")]
        {
            if let Some(ref mut port) = self.port {
                port.write(buf)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "serial port not connected"
                ))
            }
        }
        
        #[cfg(not(feature = "serial"))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "serial support not enabled"
            ))
        }
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        #[cfg(feature = "serial")]
        {
            if let Some(ref mut port) = self.port {
                port.flush()
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "serial port not connected"
                ))
            }
        }
        
        #[cfg(not(feature = "serial"))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "serial support not enabled"
            ))
        }
    }
}

impl std::fmt::Debug for Serial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Serial")
            .field("port_name", &self.port_name)
            .field("baud_rate", &self.baud_rate)
            .field("timeout", &self.timeout)
            .finish()
    }
}
