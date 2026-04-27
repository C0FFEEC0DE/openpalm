//! USB transport for Palm devices
//!
//! Uses libusb for USB communication with Palm devices.
//! Supports both bulk transfer and isochronous transfer modes.

#[cfg(feature = "usb")]
use libusb::{
    Context, Device, DeviceHandle, DeviceDescriptor, TransferType,
    request_type, Direction, RequestType, Recipient,
};
use std::io::{Read, Write};

use crate::error::{PilotError, Result};

// USB Vendor ID for Palm devices
const PALM_VENDOR_ID: u16 = 0x0830;
// USB Product IDs for various Palm devices
const PALM_PRODUCT_ID_ALL: u16 = 0x0001; // Generic Palm

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
pub struct Usb {
    /// USB context
    #[cfg(feature = "usb")]
    context: Option<Context>,
    /// Device handle
    #[cfg(feature = "usb")]
    handle: Option<DeviceHandle>,
    /// Device (for reference)
    #[cfg(feature = "usb")]
    device: Option<Device>,
    /// Parameters
    params: UsbParams,
    /// Write buffer for bulk transfers
    #[cfg(feature = "usb")]
    write_buffer: Vec<u8>,
    /// Read buffer
    #[cfg(feature = "usb")]
    read_buffer: Vec<u8>,
}

impl Usb {
    /// Create a new USB connection with parameters
    pub fn new(params: UsbParams) -> Self {
        Self {
            params,
            #[cfg(feature = "usb")]
            context: None,
            #[cfg(feature = "usb")]
            handle: None,
            #[cfg(feature = "usb")]
            device: None,
            #[cfg(feature = "usb")]
            write_buffer: Vec::new(),
            #[cfg(feature = "usb")]
            read_buffer: vec![0u8; 65536],
        }
    }
    
    /// Create with default parameters
    pub fn new_palm() -> Self {
        Self::new(UsbParams::default())
    }
    
    /// Find and open a Palm device
    pub fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "usb")]
        {
            let context = Context::new()
                .map_err(|e| PilotError::SockIo)?;
            
            let device = context.devices()
                .map_err(|e| PilotError::SockIo)?
                .iter()
                .find(|d| {
                    if let Ok(desc) = d.device_descriptor() {
                        desc.vendor_id() == self.params.vendor_id
                    } else {
                        false
                    }
                })
                .ok_or(PilotError::SockNotFound)?;
            
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
        
        #[cfg(not(feature = "usb"))]
        {
            Err(PilotError::Unimplemented)
        }
    }
    
    /// Disconnect and release resources
    pub fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "usb")]
        {
            if let Some(ref mut handle) = self.handle {
                let _ = handle.release_interface(self.params.interface);
            }
            self.handle = None;
            self.device = None;
            self.context = None;
            Ok(())
        }
        
        #[cfg(not(feature = "usb"))]
        {
            Err(PilotError::Unimplemented)
        }
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        #[cfg(feature = "usb")]
        {
            self.handle.is_some()
        }
        
        #[cfg(not(feature = "usb"))]
        {
            false
        }
    }
    
    /// Get device info
    #[cfg(feature = "usb")]
    pub fn device_info(&self) -> Option<String> {
        #[cfg(feature = "usb")]
        {
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
        
        #[cfg(not(feature = "usb"))]
        {
            None
        }
    }
}

impl Read for Usb {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        #[cfg(feature = "usb")]
        {
            let handle = self.handle.as_mut()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotConnected,
                        "USB device not connected"
                    )
                })?;
            
            if self.params.bulk_mode {
                // Bulk transfer
                let timeout = std::time::Duration::from_millis(self.params.timeout_ms);
                let bytes_read = handle.read_bulk(
                    0x81, // Bulk IN endpoint
                    buf,
                    timeout,
                ).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                })?;
                Ok(bytes_read)
            } else {
                // Interrupt transfer
                let timeout = std::time::Duration::from_millis(self.params.timeout_ms);
                let bytes_read = handle.read_interrupt(
                    0x81, // Interrupt IN endpoint
                    buf,
                    timeout,
                ).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                })?;
                Ok(bytes_read)
            }
        }
        
        #[cfg(not(feature = "usb"))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "USB support not enabled"
            ))
        }
    }
}

impl Write for Usb {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        #[cfg(feature = "usb")]
        {
            let handle = self.handle.as_mut()
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotConnected,
                        "USB device not connected"
                    )
                })?;
            
            if self.params.bulk_mode {
                // Bulk transfer
                let timeout = std::time::Duration::from_millis(self.params.timeout_ms);
                let bytes_written = handle.write_bulk(
                    0x02, // Bulk OUT endpoint
                    buf,
                    timeout,
                ).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                })?;
                Ok(bytes_written)
            } else {
                // Interrupt transfer
                let timeout = std::time::Duration::from_millis(self.params.timeout_ms);
                let bytes_written = handle.write_interrupt(
                    0x02, // Interrupt OUT endpoint
                    buf,
                    timeout,
                ).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                })?;
                Ok(bytes_written)
            }
        }
        
        #[cfg(not(feature = "usb"))]
        {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "USB support not enabled"
            ))
        }
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        // USB doesn't need explicit flush
        Ok(())
    }
}

impl std::fmt::Debug for Usb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Usb")
            .field("params", &self.params)
            .field("connected", &self.is_connected())
            .finish()
    }
}
