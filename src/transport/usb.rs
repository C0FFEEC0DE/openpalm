//! USB transport for Palm devices
//!
//! Uses libusb for USB communication with Palm devices.
//! Supports both bulk transfer and isochronous transfer modes.

use crate::error::{PilotError, Result};
use std::io::{Read, Write};

/// USB Vendor ID for Palm devices
pub const PALM_VENDOR_ID: u16 = 0x0830;
/// USB Product IDs for various Palm devices
pub const PALM_PRODUCT_ID_ALL: u16 = 0x0001;

/// USB connection parameters
#[derive(Debug, Clone)]
pub struct UsbParams {
    /// Vendor ID (default: 0x0830 for Palm)
    pub vendor_id: u16,
    /// Product ID (default: 0x0001 for all)
    pub product_id: u16,
    /// Interface number
    pub interface: u8,
    /// Alternate setting
    pub alternate: u8,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Use bulk transfer (vs interrupt)
    pub bulk_mode: bool,
}

impl Default for UsbParams {
    fn default() -> Self {
        Self {
            vendor_id: PALM_VENDOR_ID,
            product_id: PALM_PRODUCT_ID_ALL,
            interface: 0,
            alternate: 0,
            timeout_ms: 30000,
            bulk_mode: true,
        }
    }
}

/// USB connection implementation
#[cfg(feature = "usb")]
pub struct Usb {
    context: Option<libusb::Context>,
    handle: Option<libusb::DeviceHandle>,
    device: Option<libusb::Device>,
    params: UsbParams,
    read_buffer: Vec<u8>,
}

#[cfg(feature = "usb")]
impl Usb {
    /// Create a new USB connection with parameters
    pub fn new(params: UsbParams) -> Self {
        Self {
            params,
            context: None,
            handle: None,
            device: None,
            read_buffer: vec![0u8; 65536],
        }
    }
    
    /// Create with default parameters
    pub fn new_palm() -> Self {
        Self::new(UsbParams::default())
    }
    
    /// Find and open a Palm device
    pub fn connect(&mut self) -> Result<()> {
        use libusb::Context;
        
        let context = Context::new()
            .map_err(|e| PilotError::SockIo)?;
        
        let device = context.devices()
            .map_err(|e| PilotError::SockIo)?
            .iter()
            .find(|d| {
                d.device_descriptor()
                    .map(|desc| desc.vendor_id() == self.params.vendor_id)
                    .unwrap_or(false)
            })
            .ok_or(PilotError::FileNotFound)?;
        
        let mut handle = device.open()
            .map_err(|e| PilotError::SockIo)?;
        
        // Claim interface
        handle.claim_interface(self.params.interface)
            .map_err(|e| PilotError::SockIo)?;
        
        self.context = Some(context);
        self.device = Some(device);
        self.handle = Some(handle);
        
        Ok(())
    }
    
    /// Disconnect and release resources
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(ref mut handle) = self.handle {
            handle.release_interface(self.params.interface)
                .map_err(|e| PilotError::SockIo)?;
        }
        self.handle = None;
        self.device = None;
        self.context = None;
        Ok(())
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.handle.is_some()
    }
    
    /// Get device info string
    pub fn device_info(&self) -> Option<String> {
        self.device.as_ref().and_then(|d| {
            d.device_descriptor().ok().map(|desc| {
                format!(
                    "Palm Device (VID: 0x{:04X}, PID: 0x{:04X})",
                    desc.vendor_id(),
                    desc.product_id()
                )
            })
        })
    }
    
    /// Get vendor ID
    pub fn vendor_id(&self) -> u16 {
        self.params.vendor_id
    }
    
    /// Get product ID
    pub fn product_id(&self) -> u16 {
        self.params.product_id
    }
}

#[cfg(feature = "usb")]
impl Read for Usb {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let handle = self.handle.as_mut()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "USB device not connected"
                )
            })?;
        
        let timeout = std::time::Duration::from_millis(self.params.timeout_ms);
        
        if self.params.bulk_mode {
            handle.read_bulk(0x81, buf, timeout)
        } else {
            handle.read_interrupt(0x81, buf, timeout)
        }.map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })
    }
}

#[cfg(feature = "usb")]
impl Write for Usb {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let handle = self.handle.as_mut()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "USB device not connected"
                )
            })?;
        
        let timeout = std::time::Duration::from_millis(self.params.timeout_ms);
        
        if self.params.bulk_mode {
            handle.write_bulk(0x02, buf, timeout)
        } else {
            handle.write_interrupt(0x02, buf, timeout)
        }.map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        // USB doesn't need explicit flush
        Ok(())
    }
}

#[cfg(feature = "usb")]
impl std::fmt::Debug for Usb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Usb")
            .field("params", &self.params)
            .field("connected", &self.is_connected())
            .finish()
    }
}

#[cfg(feature = "usb")]
impl Drop for Usb {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

// Stub implementation when USB feature is disabled
#[cfg(not(feature = "usb"))]
pub struct Usb {
    params: UsbParams,
    connected: bool,
}

#[cfg(not(feature = "usb"))]
impl Usb {
    /// Create a new USB connection with parameters
    pub fn new(params: UsbParams) -> Self {
        Self {
            params,
            connected: false,
        }
    }
    
    /// Create with default parameters
    pub fn new_palm() -> Self {
        Self::new(UsbParams::default())
    }
    
    /// Connect to USB device (not available without usb feature)
    pub fn connect(&mut self) -> Result<()> {
        Err(PilotError::Unimplemented)
    }
    
    /// Disconnect from USB device
    pub fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Get device info
    pub fn device_info(&self) -> Option<String> {
        None
    }
    
    /// Get vendor ID
    pub fn vendor_id(&self) -> u16 {
        self.params.vendor_id
    }
    
    /// Get product ID
    pub fn product_id(&self) -> u16 {
        self.params.product_id
    }
}

#[cfg(not(feature = "usb"))]
impl std::fmt::Debug for Usb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Usb")
            .field("params", &self.params)
            .field("connected", &self.connected)
            .field("usb_feature", &"disabled")
            .finish()
    }
}

#[cfg(not(feature = "usb"))]
impl Read for Usb {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "USB feature not enabled"
        ))
    }
}

#[cfg(not(feature = "usb"))]
impl Write for Usb {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "USB feature not enabled"
        ))
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}