//! USB transport for Palm devices
//!
//! Uses libusb1-sys (raw FFI) for USB communication with Palm devices.
//! Supports USB bulk transfer mode.

use crate::error::{PilotError, Result};
use crate::transport::{Connection, ConnectionState};
use libusb1_sys as libusb;
use std::io::{Read, Write};
use std::time::Duration;

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
pub struct Usb {
    context: Option<*mut libusb::libusb_context>,
    handle: Option<*mut libusb::libusb_device_handle>,
    device_info_str: Option<String>,
    params: UsbParams,
    _not_send: std::marker::PhantomData<*const ()>,
    endpoint_in: u8,
    endpoint_out: u8,
}

impl Usb {
    /// Create a new USB connection with parameters
    pub fn new(params: UsbParams) -> Self {
        Self {
            params,
            context: None,
            handle: None,
            device_info_str: None,
            _not_send: std::marker::PhantomData,
            endpoint_in: 0x81,   // default Palm bulk IN endpoint
            endpoint_out: 0x02,  // default Palm bulk OUT endpoint
        }
    }

    /// Create with default parameters
    pub fn new_palm() -> Self {
        Self::new(UsbParams::default())
    }

    /// Set timeout for operations
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.params.timeout_ms = timeout.as_millis() as u64;
    }

    /// Find and open a Palm device
    pub fn connect(&mut self) -> Result<()> {
        unsafe {
            let mut ctx: *mut libusb::libusb_context = std::ptr::null_mut();
            let ret = libusb::libusb_init(&mut ctx);
            if ret < 0 {
                return Err(PilotError::SockIo);
            }
            self.context = Some(ctx);

            let mut dev_list: *const *mut libusb::libusb_device = std::ptr::null();
            let count = libusb::libusb_get_device_list(ctx, &mut dev_list);
            if count < 0 {
                libusb::libusb_exit(ctx);
                self.context = None;
                return Err(PilotError::SockIo);
            }

            let mut found_handle: *mut libusb::libusb_device_handle = std::ptr::null_mut();
            let mut found_info: Option<String> = None;

            for i in 0..count {
                let dev = *dev_list.offset(i);
                let mut desc: libusb::libusb_device_descriptor = std::mem::zeroed();
                let ret = libusb::libusb_get_device_descriptor(dev, &mut desc);
                if ret < 0 {
                    continue;
                }
                if desc.idVendor == self.params.vendor_id {
                    let ret = libusb::libusb_open(dev, &mut found_handle);
                    if ret == 0 {
                        let ret = libusb::libusb_claim_interface(
                            found_handle,
                            self.params.interface as i32,
                        );
                        if ret < 0 {
                            libusb::libusb_close(found_handle);
                            found_handle = std::ptr::null_mut();
                            continue;
                        }
                        found_info = Some(format!(
                            "Palm Device (VID: 0x{:04X}, PID: 0x{:04X})",
                            desc.idVendor, desc.idProduct
                        ));

                        // Discover bulk endpoint addresses from active config descriptor
                        let mut config_desc: *const libusb::libusb_config_descriptor = std::ptr::null();
                        if libusb::libusb_get_active_config_descriptor(dev, &mut config_desc) == 0
                            && !config_desc.is_null()
                        {
                            let cfg = &*config_desc;
                            for if_idx in 0..cfg.bNumInterfaces as isize {
                                let iface = &*cfg.interface.offset(if_idx);
                                for altset_idx in 0..iface.num_altsetting {
                                    let altset = &*iface.altsetting.offset(altset_idx as isize);
                                    for ep_idx in 0..altset.bNumEndpoints as isize {
                                        let ep = &*altset.endpoint.offset(ep_idx);
                                        // LIBUSB_TRANSFER_TYPE_BULK = 2
                                        if (ep.bmAttributes & 0x03) == 0x02 {
                                            if ep.bEndpointAddress & 0x80 != 0 {
                                                self.endpoint_in = ep.bEndpointAddress;
                                            } else {
                                                self.endpoint_out = ep.bEndpointAddress;
                                            }
                                        }
                                    }
                                }
                            }
                            libusb::libusb_free_config_descriptor(config_desc);
                        }

                        break;
                    }
                }
            }

            libusb::libusb_free_device_list(dev_list, 1);

            if found_handle.is_null() {
                libusb::libusb_exit(ctx);
                self.context = None;
                return Err(PilotError::FileNotFound);
            }

            self.handle = Some(found_handle);
            self.device_info_str = found_info;
            Ok(())
        }
    }

    /// Close the device and release resources
    pub fn disconnect(&mut self) -> Result<()> {
        unsafe {
            if let Some(handle) = self.handle {
                libusb::libusb_release_interface(handle, self.params.interface as i32);
                libusb::libusb_close(handle);
            }
            if let Some(ctx) = self.context {
                libusb::libusb_exit(ctx);
            }
        }
        self.handle = None;
        self.context = None;
        self.device_info_str = None;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.handle.is_some()
    }

    /// Get device info string
    pub fn device_info(&self) -> Option<String> {
        self.device_info_str.clone()
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

impl Read for Usb {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let handle = self.handle.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "USB device not connected")
        })?;

        unsafe {
            let mut transferred: i32 = 0;
            let ret = libusb::libusb_bulk_transfer(
                handle,
                self.endpoint_in,
                buf.as_mut_ptr(),
                buf.len() as i32,
                &mut transferred,
                self.params.timeout_ms as u32,
            );
            if ret < 0 {
                return Err(std::io::Error::other(
                    format!("USB read error: {}", ret),
                ));
            }
            Ok(transferred as usize)
        }
    }
}

impl Write for Usb {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let handle = self.handle.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "USB device not connected")
        })?;

        unsafe {
            let mut transferred: i32 = 0;
            let ret = libusb::libusb_bulk_transfer(
                handle,
                self.endpoint_out,
                // SAFETY: The const-to-mutable cast is safe here because
                // libusb_bulk_transfer does not modify the buffer for OUT
                // endpoints; the pointer is only used for reading data to send.
                buf.as_ptr() as *mut u8,
                buf.len() as i32,
                &mut transferred,
                self.params.timeout_ms as u32,
            );
            if ret < 0 {
                return Err(std::io::Error::other(
                    format!("USB write error: {}", ret),
                ));
            }
            Ok(transferred as usize)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for Usb {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

// SAFETY: libusb context and device handle pointers are safe to Send because
// each Usb instance owns its own isolated context/handle pair. libusb is
// designed for multi-threaded use and no other thread can access these pointers.
unsafe impl Send for Usb {}

impl Connection for Usb {
    fn connect(&mut self) -> Result<()> {
        self.connect()
    }

    fn disconnect(&mut self) -> Result<()> {
        self.disconnect()
    }

    fn is_connected(&self) -> bool {
        self.is_connected()
    }

    fn set_timeout(&mut self, timeout: Duration) {
        self.set_timeout(timeout)
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
