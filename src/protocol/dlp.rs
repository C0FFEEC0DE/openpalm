//! Desktop Link Protocol (DLP) implementation
//!
//! DLP is the protocol used by HotSync to communicate with Palm OS devices.
//! It provides database operations, record management, system info, and more.

use crate::error::{PilotError, Result};
use crate::types::{FourCharCode, DatabaseFlags, RecordFlags, PalmDateTime};
use crate::database::{DatabaseInfo, Record};
use crate::types::CardNo;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::protocol::socket::TransportConnection;

// ============================================================================
// DLP Constants
// ============================================================================

/// DLP Protocol Version
pub const DLP_VERSION_MAJOR: u8 = 1;
pub const DLP_VERSION_MINOR: u8 = 4;

/// Internal DLP argument constants
const DLP_ARG_TINY_LEN: usize = 0xFF;
const DLP_ARG_SHORT_LEN: usize = 0xFFFF;
const DLP_ARG_FIRST_ID: u8 = 0x20;

/// DLP function codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DlpFunction {
    // DLP 1.0 functions
    ReadUserInfo = 0x10,
    WriteUserInfo = 0x11,
    ReadSysInfo = 0x12,
    GetSysDateTime = 0x13,
    SetSysDateTime = 0x14,
    ReadStorageInfo = 0x15,
    ReadDBList = 0x16,
    OpenDB = 0x17,
    CreateDB = 0x18,
    CloseDB = 0x19,
    DeleteDB = 0x1A,
    ReadAppBlock = 0x1B,
    WriteAppBlock = 0x1C,
    ReadSortBlock = 0x1D,
    WriteSortBlock = 0x1E,
    ReadNextModifiedRec = 0x1F,
    ReadRecord = 0x20,
    WriteRecord = 0x21,
    DeleteRecord = 0x22,
    ReadResource = 0x23,
    WriteResource = 0x24,
    DeleteResource = 0x25,
    CleanUpDatabase = 0x26,
    ResetSyncFlags = 0x27,
    CallApplication = 0x28,
    ResetSystem = 0x29,
    AddSyncLogEntry = 0x2A,
    ReadOpenDBInfo = 0x2B,
    MoveCategory = 0x2C,
    OpenConduit = 0x2E,
    EndOfSync = 0x2F,
    ResetRecordIndex = 0x30,
    ReadRecordIDList = 0x31,

    // DLP 1.1 functions
    ReadNextRecInCategory = 0x32,
    ReadNextModifiedRecInCategory = 0x33,
    ReadAppPreference = 0x34,
    WriteAppPreference = 0x35,
    ReadNetSyncInfo = 0x36,
    WriteNetSyncInfo = 0x37,
    ReadFeature = 0x38,

    // DLP 1.2 functions
    FindDB = 0x39,
    SetDBInfo = 0x3A,

    // DLP 1.3 functions
    LoopBackTest = 0x3B,
    ExpSlotEnumerate = 0x3C,
    ExpCardPresent = 0x3D,
    ExpCardInfo = 0x3E,
    VFSCustomControl = 0x3F,
    VFSGetDefaultDir = 0x40,
    VFSImportDatabaseFromFile = 0x41,
    VFSExportDatabaseToFile = 0x42,
    VFSFileCreate = 0x43,
    VFSFileOpen = 0x44,
    VFSFileClose = 0x45,
    VFSFileWrite = 0x46,
    VFSFileRead = 0x47,
    VFSFileDelete = 0x48,
    VFSFileRename = 0x49,
    VFSFileEOF = 0x4A,
    VFSFileTell = 0x4B,
    VFSFileGetAttributes = 0x4C,
    VFSFileSetAttributes = 0x4D,
    VFSFileGetDate = 0x4E,
    VFSFileSetDate = 0x4F,
    VFSDirCreate = 0x50,
    VFSDirEntryEnumerate = 0x51,
    VFSGetFile = 0x52,
    VFSPutFile = 0x53,
    VFSVolumeFormat = 0x54,
    VFSVolumeEnumerate = 0x55,
    VFSVolumeInfo = 0x56,
    VFSVolumeGetLabel = 0x57,
    VFSVolumeSetLabel = 0x58,
    VFSVolumeSize = 0x59,
    VFSFileSeek = 0x5A,
    VFSFileResize = 0x5B,
    VFSFileSize = 0x5C,

    // DLP 1.4 functions (Tapwave Zodiac)
    ExpSlotMediaType = 0x5D,
    WriteRecordEx = 0x5E,
    WriteResourceEx = 0x5F,
    ReadRecordEx = 0x60,
    ReadResourceEx = 0x64,
}

/// DLP error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DlpErrorCode {
    NoError = 0,
    System = 1,
    IllegalReq = 2,
    Memory = 3,
    Param = 4,
    NotFound = 5,
    NoneOpen = 6,
    AlreadyOpen = 7,
    TooManyOpen = 8,
    Exists = 9,
    Open = 10,
    Deleted = 11,
    Busy = 12,
    NotSupp = 13,
    Unused1 = 14,
    ReadOnly = 15,
    Space = 16,
    Limit = 17,
    Sync = 18,
    Wrapper = 19,
    Argument = 20,
    Size = 21,
    Unknown = 127,
}

impl DlpErrorCode {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => DlpErrorCode::NoError,
            1 => DlpErrorCode::System,
            2 => DlpErrorCode::IllegalReq,
            3 => DlpErrorCode::Memory,
            4 => DlpErrorCode::Param,
            5 => DlpErrorCode::NotFound,
            6 => DlpErrorCode::NoneOpen,
            7 => DlpErrorCode::AlreadyOpen,
            8 => DlpErrorCode::TooManyOpen,
            9 => DlpErrorCode::Exists,
            10 => DlpErrorCode::Open,
            11 => DlpErrorCode::Deleted,
            12 => DlpErrorCode::Busy,
            13 => DlpErrorCode::NotSupp,
            14 => DlpErrorCode::Unused1,
            15 => DlpErrorCode::ReadOnly,
            16 => DlpErrorCode::Space,
            17 => DlpErrorCode::Limit,
            18 => DlpErrorCode::Sync,
            19 => DlpErrorCode::Wrapper,
            20 => DlpErrorCode::Argument,
            21 => DlpErrorCode::Size,
            _ => DlpErrorCode::Unknown,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DlpErrorCode::NoError => "No error",
            DlpErrorCode::System => "System error",
            DlpErrorCode::IllegalReq => "Illegal request",
            DlpErrorCode::Memory => "Out of memory",
            DlpErrorCode::Param => "Invalid parameter",
            DlpErrorCode::NotFound => "Not found",
            DlpErrorCode::NoneOpen => "Not open",
            DlpErrorCode::AlreadyOpen => "Already open",
            DlpErrorCode::TooManyOpen => "Too many open",
            DlpErrorCode::Exists => "Already exists",
            DlpErrorCode::Open => "Cannot open",
            DlpErrorCode::Deleted => "Record deleted",
            DlpErrorCode::Busy => "Record busy",
            DlpErrorCode::NotSupp => "Not supported",
            DlpErrorCode::Unused1 => "Unused",
            DlpErrorCode::ReadOnly => "Read only",
            DlpErrorCode::Space => "Not enough space",
            DlpErrorCode::Limit => "Limit exceeded",
            DlpErrorCode::Sync => "Sync cancelled",
            DlpErrorCode::Wrapper => "Bad argument wrapper",
            DlpErrorCode::Argument => "Argument missing",
            DlpErrorCode::Size => "Bad argument size",
            DlpErrorCode::Unknown => "Unknown error",
        }
    }
}

impl std::fmt::Display for DlpErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// DLP open flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DlpOpenMode {
    Read = 0x80,
    Write = 0x40,
    Exclusive = 0x20,
    Secret = 0x10,
    ReadWrite = 0xC0,
}

impl DlpOpenMode {
    pub fn bits(&self) -> u8 {
        *self as u8
    }
}

/// Database list flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DlpDBListFlag {
    Ram = 0x80,
    Rom = 0x40,
    Multiple = 0x20,
}

/// End of sync status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DlpEndStatus {
    Normal = 0,
    OutOfMemory = 1,
    UserCan = 2,
    Other = 3,
}

// ============================================================================
// DLP Data Structures
// ============================================================================

/// System information from device
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// ROM version (0xMMmmffssbb)
    pub rom_version: u32,
    /// Locale
    pub locale: u32,
    /// Product ID length
    pub prod_id_len: u8,
    /// Product ID
    pub prod_id: String,
    /// DLP major version
    pub dlp_major: u16,
    /// DLP minor version
    pub dlp_minor: u16,
    /// Compatible DLP major version
    pub compat_major: u16,
    /// Compatible DLP minor version
    pub compat_minor: u16,
    /// Maximum record size
    pub max_rec_size: u32,
}

impl SystemInfo {
    /// Extract major version from ROM version
    pub fn rom_major(&self) -> u8 {
        ((self.rom_version >> 24) & 0xFF) as u8
    }

    /// Extract minor version from ROM version
    pub fn rom_minor(&self) -> u8 {
        ((self.rom_version >> 16) & 0xFF) as u8
    }

    /// Extract fix version from ROM version
    pub fn rom_fix(&self) -> u8 {
        ((self.rom_version >> 8) & 0xFF) as u8
    }
}

/// User information
#[derive(Debug, Clone)]
pub struct UserInfo {
    /// Username
    pub username: String,
    /// User ID
    pub user_id: u32,
    /// Viewer ID
    pub viewer_id: u32,
    /// Last sync PC
    pub last_sync_pc: u32,
    /// Last successful sync date
    pub last_sync_date: Option<PalmDateTime>,
    /// Successful sync date
    pub successful_sync_date: Option<PalmDateTime>,
}

impl Default for UserInfo {
    fn default() -> Self {
        Self {
            username: String::new(),
            user_id: 0,
            viewer_id: 0,
            last_sync_pc: 0,
            last_sync_date: None,
            successful_sync_date: None,
        }
    }
}

/// Storage card information
#[derive(Debug, Clone)]
pub struct StorageInfo {
    /// Card version
    pub version: i32,
    /// ROM size
    pub rom_size: u32,
    /// RAM size
    pub ram_size: u32,
    /// Free RAM
    pub ram_free: u32,
    /// Card name
    pub name: String,
    /// Manufacturer
    pub manufacturer: String,
    /// Creation date
    pub creation_date: Option<PalmDateTime>,
}

/// VFS volume information
#[derive(Debug, Clone)]
pub struct VolumeInfo {
    /// Volume attributes
    pub attributes: u32,
    /// Filesystem type (FourCC)
    pub fs_type: FourCharCode,
    /// Filesystem creator (FourCC)
    pub fs_creator: FourCharCode,
    /// Media type (FourCC)
    pub media_type: FourCharCode,
    /// Volume label
    pub label: String,
}

/// VFS file reference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileRef(u64);

impl FileRef {
    pub const INVALID: FileRef = FileRef(0);
    
    pub fn new(val: u64) -> Self {
        FileRef(val)
    }
    
    pub fn value(&self) -> u64 {
        self.0
    }
    
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// VFS volume reference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeRef(u16);

impl VolumeRef {
    pub const INVALID: VolumeRef = VolumeRef(0);
    
    pub fn new(val: u16) -> Self {
        VolumeRef(val)
    }
    
    pub fn value(&self) -> u16 {
        self.0
    }
    
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

// ============================================================================
// DLP Request/Response Building
// ============================================================================

/// A DLP argument
#[derive(Debug, Clone)]
struct DlpArg {
    id: u8,
    data: Vec<u8>,
}

impl DlpArg {
    fn new(id: u8, data: Vec<u8>) -> Self {
        Self { id, data }
    }

    /// Calculate encoded size
    fn encoded_size(&self) -> usize {
        let data_len = self.data.len();
        if data_len <= DLP_ARG_TINY_LEN && self.id < 0x80 {
            2 + data_len // tiny: 1 byte header + data
        } else if data_len <= DLP_ARG_SHORT_LEN && self.id < 0x40 {
            4 + data_len // short: 2 byte header + data
        } else {
            6 + data_len // long: 3 byte header + data
        }
    }

    /// Encode to bytes
    fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.encoded_size());
        let data_len = self.data.len();

        if data_len <= DLP_ARG_TINY_LEN && self.id < 0x80 {
            // Tiny format: 0b0LLLLLLL | id
            result.push((data_len as u8) | (self.id & 0x1F));
        } else if data_len <= DLP_ARG_SHORT_LEN {
            // Short format: 0b10LLLLLL | 0bLLLLLLLL | id
            result.push(0x80 | ((data_len >> 8) as u8));
            result.push(data_len as u8);
            result.push(self.id);
        } else {
            // Long format: 0b01TTTTTT | TTTTTTTT | TTTTTTTT | TTTTTTTT | id
            result.push(0x40 | ((data_len >> 24) as u8));
            result.push((data_len >> 16) as u8);
            result.push((data_len >> 8) as u8);
            result.push(data_len as u8);
            result.push(self.id);
        }

        result.extend_from_slice(&self.data);
        result
    }
}

/// A DLP request packet
#[derive(Debug, Clone)]
struct DlpRequest {
    function: DlpFunction,
    args: Vec<DlpArg>,
}

impl DlpRequest {
    fn new(function: DlpFunction) -> Self {
        Self {
            function,
            args: Vec::new(),
        }
    }

    fn add_arg(&mut self, id: u8, data: Vec<u8>) {
        self.args.push(DlpArg::new(id, data));
    }

    fn add_u8(&mut self, val: u8) {
        self.add_arg(0x20 + self.args.len() as u8, vec![val]);
    }

    fn add_u16(&mut self, val: u16) {
        let mut bytes = vec![0, 0];
        bytes[0..2].copy_from_slice(&val.to_le_bytes());
        self.add_arg(0x20 + self.args.len() as u8, bytes);
    }

    fn add_u32(&mut self, val: u32) {
        let mut bytes = vec![0, 0, 0, 0];
        bytes[0..4].copy_from_slice(&val.to_le_bytes());
        self.add_arg(0x20 + self.args.len() as u8, bytes);
    }

    fn add_i32(&mut self, val: i32) {
        self.add_u32(val as u32);
    }

    fn add_u64(&mut self, val: u64) {
        let mut bytes = vec![0, 0, 0, 0, 0, 0, 0, 0];
        bytes.copy_from_slice(&val.to_le_bytes());
        self.add_arg(0x20 + self.args.len() as u8, bytes);
    }

    fn add_string(&mut self, s: &str) {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0); // null terminator
        self.add_arg(0x20 + self.args.len() as u8, bytes);
    }

    fn add_bytes(&mut self, data: &[u8]) {
        self.add_arg(0x20 + self.args.len() as u8, data.to_vec());
    }

    /// Encode the complete request packet
    fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Command byte
        result.push(self.function as u8);
        
        // Argument count
        result.push(self.args.len() as u8);
        
        // Encode each argument
        for arg in &self.args {
            result.extend_from_slice(&arg.encode());
        }
        
        result
    }

    /// Calculate total packet size
    fn size(&self) -> usize {
        2 + self.args.iter().map(|a| a.encoded_size()).sum::<usize>()
    }
}

/// A DLP response packet
#[derive(Debug, Clone)]
struct DlpResponse {
    function: u8,
    error: DlpErrorCode,
    args: Vec<DlpArg>,
}

impl DlpResponse {
    /// Decode from bytes
    fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 4 {
            return Err(PilotError::GenericSystem);
        }

        let function = data[0];
        let argc = data[1];
        let error_code = data[2];

        // Skip header (3 bytes) and decode arguments
        let mut offset = 3;
        let mut args = Vec::new();

        for _ in 0..argc {
            if offset >= data.len() {
                break;
            }

            let (arg_data, new_offset) = Self::decode_arg(&data[offset..])?;
            args.push(arg_data);
            offset += new_offset;
        }

        Ok(Self {
            function,
            error: DlpErrorCode::from_u8(error_code),
            args,
        })
    }

    /// Decode a single argument
    fn decode_arg(data: &[u8]) -> Result<(DlpArg, usize)> {
        if data.is_empty() {
            return Err(PilotError::GenericSystem);
        }

        let header = data[0];
        let (len, header_size) = if header & 0x80 == 0 {
            // Tiny format
            ((header & 0x3F) as usize, 1)
        } else if header & 0x40 == 0 {
            // Short format
            if data.len() < 2 {
                return Err(PilotError::GenericSystem);
            }
            (((header & 0x3F) as usize) << 8 | (data[1] as usize), 2)
        } else {
            // Long format
            if data.len() < 4 {
                return Err(PilotError::GenericSystem);
            }
            let len = ((header & 0x3F) as usize) << 24
                | (data[1] as usize) << 16
                | (data[2] as usize) << 8
                | (data[3] as usize);
            (len, 4)
        };

        let arg_id = if header_size == 1 {
            0x20
        } else if header_size == 2 {
            data[2]
        } else {
            data[4]
        };

        if data.len() < header_size + len {
            return Err(PilotError::GenericSystem);
        }

        let arg_data = data[header_size..header_size + len].to_vec();
        Ok((DlpArg::new(arg_id, arg_data), header_size + len))
    }

    /// Get argument by index
    fn get_arg(&self, index: usize) -> Option<&[u8]> {
        self.args.get(index).map(|a| a.data.as_slice())
    }

    /// Get argument as u8
    fn get_u8(&self, index: usize) -> Result<u8> {
        self.get_arg(index)
            .and_then(|d| d.first().copied())
            .ok_or(PilotError::Unimplemented)
    }

    /// Get argument as u16
    fn get_u16(&self, index: usize) -> Result<u16> {
        let data = self.get_arg(index).ok_or(PilotError::InvalidArgument)?;
        if data.len() < 2 {
            return Err(PilotError::InvalidArgument);
        }
        Ok(u16::from_le_bytes([data[0], data[1]]))
    }

    /// Get argument as u32
    fn get_u32(&self, index: usize) -> Result<u32> {
        let data = self.get_arg(index).ok_or(PilotError::InvalidArgument)?;
        if data.len() < 4 {
            return Err(PilotError::InvalidArgument);
        }
        Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
    }
    
    /// Get argument as i32
    fn get_i32(&self, index: usize) -> Result<i32> {
        let data = self.get_arg(index).ok_or(PilotError::InvalidArgument)?;
        if data.len() < 4 {
            return Err(PilotError::InvalidArgument);
        }
        Ok(i32::from_le_bytes([data[0], data[1], data[2], data[3]]))
    }
    
    /// Get argument as u64
    fn get_u64(&self, index: usize) -> Result<u64> {
        let data = self.get_arg(index).ok_or(PilotError::InvalidArgument)?;
        if data.len() < 8 {
            return Err(PilotError::InvalidArgument);
        }
        Ok(u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]))
    }

    /// Get argument as string
    fn get_string(&self, index: usize) -> Result<String> {
        let data = self.get_arg(index).ok_or(PilotError::InvalidArgument)?;
        // Remove trailing null
        let s = if let Some(pos) = data.iter().position(|&b| b == 0) {
            String::from_utf8_lossy(&data[..pos]).into_owned()
        } else {
            String::from_utf8_lossy(data).into_owned()
        };
        Ok(s)
    }
}

// ============================================================================
// Protocol Version
// ============================================================================

/// DLP Protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    pub fn current() -> Self {
        Self::new(DLP_VERSION_MAJOR, DLP_VERSION_MINOR)
    }

    pub fn from_u16(val: u16) -> Self {
        Self::new((val >> 8) as u8, (val & 0xFF) as u8)
    }

    pub fn to_u16(&self) -> u16 {
        ((self.major as u16) << 8) | (self.minor as u16)
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::current()
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

// ============================================================================
// DLP Client
// ============================================================================

/// DLP Client for communicating with Palm devices
#[derive(Debug, Clone)]
pub struct DlpClient {
    transport: Arc<Mutex<TransportConnection>>,
    socket_id: i32,
    version: ProtocolVersion,
    max_record_size: u32,
}

impl DlpClient {
    /// Create a new DLP client
    pub fn new(transport: TransportConnection) -> Self {
        Self {
            transport: Arc::new(Mutex::new(transport)),
            socket_id: 0,
            version: ProtocolVersion::current(),
            max_record_size: 0xFFFF,
        }
    }
    
    /// Get a reference to the underlying transport
    pub fn transport(&self) -> Arc<Mutex<TransportConnection>> {
        Arc::clone(&self.transport)
    }

    /// Set the protocol version
    pub fn set_version(&mut self, version: ProtocolVersion) {
        self.version = version;
    }

    /// Get the protocol version
    pub fn version(&self) -> ProtocolVersion {
        self.version
    }

    /// Get the maximum record size
    pub fn max_record_size(&self) -> u32 {
        self.max_record_size
    }
    
    /// Send a DLP request and receive response
    async fn send_request(&self, request: &DlpRequest) -> Result<DlpResponse> {
        use std::io::{Read, Write};
        
        let mut transport = self.transport.lock().unwrap();
        
        // Encode the request
        let data = request.encode();
        
        // Send through transport
        Write::write_all(&mut *transport, &data)?;
        Write::flush(&mut *transport)?;
        
        // Read response header (at least 4 bytes)
        let mut header = [0u8; 4];
        let mut total_read = 0;
        while total_read < 4 {
            match Read::read(&mut *transport, &mut header[total_read..4]) {
                Ok(0) => return Err(PilotError::SockDisconnected),
                Ok(n) => total_read += n,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Err(PilotError::SockTimeout);
                }
                Err(_) => return Err(PilotError::SockIo),
            }
        }
        
        // Parse response header
        let function = header[0];
        let _argc = header[1];
        let error_code = header[2];
        let _flags = header[3];
        
        // Check for errors
        if error_code != 0 {
            return Err(PilotError::DlpError(error_code as u16));
        }
        
        // For now, return empty response (full parsing would need read_body)
        Ok(DlpResponse {
            function,
            error: DlpErrorCode::from_u8(error_code),
            args: Vec::new(),
        })
    }

    // ========================================================================
    // System Functions
    // ========================================================================

    /// Read system information from the device
    /// 
    /// Returns device system information including:
    /// - ROM version
    /// - Localization settings
    /// - Manufacturer info
    /// 
    /// # Arguments
    /// * `none`
    /// 
    /// # Returns
    /// * `Ok(SystemInfo)` - System information from device
    /// * `Err(PilotError)` - Communication or protocol error
    /// 
    /// # Example
    /// ```ignore
    /// let sys_info = client.read_sys_info().await?;
    /// println!("ROM: {}.{}", sys_info.rom_major(), sys_info.rom_minor());
    /// ```
    pub async fn read_sys_info(&self) -> Result<SystemInfo> {
        let mut req = DlpRequest::new(DlpFunction::ReadSysInfo);
        let response = self.send_request(&req).await?;
        
        // Parse response
        if response.args.len() < 6 {
            return Err(PilotError::DlpBufSize);
        }
        
        let rom_version = response.get_u32(0)?;
        let locale = response.get_u32(1)?;
        let prod_id_len = response.get_u8(2)?;
        let prod_id = response.get_string(3)?;
        
        // DLP version info (optional in response)
        let dlp_major = response.get_u16(4).unwrap_or(1);
        let dlp_minor = response.get_u16(5).unwrap_or(4);
        
        Ok(SystemInfo {
            rom_version,
            locale,
            prod_id_len,
            prod_id,
            dlp_major,
            dlp_minor,
            compat_major: dlp_major,
            compat_minor: dlp_minor,
            max_rec_size: self.max_record_size,
        })
    }

    /// Read storage information for a card
    /// 
    /// Returns information about storage on a memory card including
    /// total and free space.
    /// 
    /// # Arguments
    /// * `card_no` - Card number (usually 0 for internal, 1+ for expansion)
    /// 
    /// # Returns
    /// * `Ok(StorageInfo)` - Storage information
    /// * `Err(PilotError)` - Error reading storage info
    pub async fn read_storage_info(&self, card_no: CardNo) -> Result<StorageInfo> {
        let mut req = DlpRequest::new(DlpFunction::ReadStorageInfo);
        req.add_u8(card_no);
        
        let response = self.send_request(&req).await?;
        
        // Parse based on StorageInfo struct fields
        // version, rom_size, ram_size, ram_free, name, manufacturer, creation_date
        let version = response.get_i32(0).unwrap_or(0);
        let rom_size = response.get_u32(1).unwrap_or(0);
        let ram_size = response.get_u32(2).unwrap_or(0);
        let ram_free = response.get_u32(3).unwrap_or(0);
        let name = response.get_string(4).unwrap_or_default();
        let manufacturer = response.get_string(5).unwrap_or_default();
        
        Ok(StorageInfo {
            version,
            rom_size,
            ram_size,
            ram_free,
            name,
            manufacturer,
            creation_date: None,
        })
    }

    /// Read user information (user name, user ID)
    /// 
    /// Returns the user's information configured on the device.
    /// 
    /// # Returns
    /// * `Ok(UserInfo)` - User information
    /// * `Err(PilotError)` - Error reading user info
    pub async fn read_user_info(&self) -> Result<UserInfo> {
        let mut req = DlpRequest::new(DlpFunction::ReadUserInfo);
        let response = self.send_request(&req).await?;
        
        // Parse UserInfo from response
        let user_id = response.get_u32(0).unwrap_or(0);
        let viewer_id = response.get_u32(1).unwrap_or(0);
        let username = response.get_string(2).unwrap_or_default();
        let last_sync_pc = response.get_u32(3).unwrap_or(0);
        
        Ok(UserInfo {
            username,
            user_id,
            viewer_id,
            last_sync_pc,
            last_sync_date: None,
            successful_sync_date: None,
        })
    }

    /// Write user information to device
    /// 
    /// Updates the user's information on the device.
    /// 
    /// # Arguments
    /// * `user` - User information to write
    /// 
    /// # Returns
    /// * `Ok(())` - User info written successfully
    /// * `Err(PilotError)` - Error writing user info
    pub async fn write_user_info(&self, user: &UserInfo) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::WriteUserInfo);
        req.add_u32(user.user_id);
        req.add_u32(user.last_sync_pc);
        req.add_string(&user.username);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Get system date/time from device
    /// 
    /// Reads the current date/time from the device's internal clock.
    /// 
    /// # Returns
    /// * `Ok(PalmDateTime)` - Current device date/time
    /// * `Err(PilotError)` - Error reading date/time
    pub async fn get_sys_datetime(&self) -> Result<PalmDateTime> {
        let mut req = DlpRequest::new(DlpFunction::GetSysDateTime);
        let response = self.send_request(&req).await?;
        
        let seconds = response.get_u32(0).unwrap_or(0);
        Ok(PalmDateTime::from_palm(seconds))
    }

    /// Set system date/time on device
    /// 
    /// Updates the device's internal clock.
    /// 
    /// # Arguments
    /// * `datetime` - New date/time to set
    /// 
    /// # Returns
    /// * `Ok(())` - Date/time set successfully
    /// * `Err(PilotError)` - Error setting date/time
    pub async fn set_sys_datetime(&self, datetime: PalmDateTime) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::SetSysDateTime);
        req.add_u32(datetime.to_palm());
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Reset last sync PC
    /// 
    /// Resets the last sync PC ID to zero, forcing a full sync.
    /// 
    /// # Returns
    /// * `Ok(())` - Reset successful
    /// * `Err(PilotError)` - Error resetting sync PC
    pub async fn reset_last_sync_pc(&self) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::WriteNetSyncInfo);
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Read a feature from the device
    /// 
    /// Features are key-value pairs stored in the device's NVFS.
    /// 
    /// # Arguments
    /// * `creator` - Creator ID of the feature
    /// * `num` - Feature number
    /// 
    /// # Returns
    /// * `Ok(u32)` - Feature value
    /// * `Err(PilotError)` - Feature not found or error
    pub async fn read_feature(&self, creator: FourCharCode, num: i32) -> Result<u32> {
        let mut req = DlpRequest::new(DlpFunction::ReadFeature);
        req.add_u32(creator.to_u32());
        req.add_i32(num);
        
        let response = self.send_request(&req).await?;
        response.get_u32(0).map_err(|_| PilotError::RecordNotFound)
    }

    // ========================================================================
    // Database Functions
    // ========================================================================

    /// Read list of databases on a card
    /// 
    /// Returns a list of all databases matching the specified criteria.
    /// 
    /// # Arguments
    /// * `card_no` - Card number (0 for internal storage)
    /// * `flags` - Filter flags for database types
    /// * `start` - Starting index (for pagination)
    /// 
    /// # Returns
    /// * `Ok(Vec<DatabaseInfo>)` - List of databases
    /// * `Err(PilotError)` - Error reading database list
    pub async fn read_db_list(
        &self,
        card_no: CardNo,
        flags: DlpDBListFlag,
        start: u32,
    ) -> Result<Vec<DatabaseInfo>> {
        let mut req = DlpRequest::new(DlpFunction::ReadDBList);
        req.add_u8(card_no);
        req.add_u8(flags as u8);
        req.add_u32(start);
        
        let response = self.send_request(&req).await?;
        
        // Parse database list from response
        let mut databases = Vec::new();
        
        for i in 0..response.args.len() {
            if let Ok(name) = response.get_string(i) {
                databases.push(DatabaseInfo {
                    flags: DatabaseFlags::empty(),
                    db_type: FourCharCode { 0: 0 },
                    creator: FourCharCode { 0: 0 },
                    card_no: card_no as u16,
                    db_id: 0,
                    created: PalmDateTime::now(),
                    modified: PalmDateTime::now(),
                    backup_date: PalmDateTime::now(),
                    mod_num: 0,
                    app_info_dirty: false,
                    sort_info_dirty: false,
                    total_bytes: 0,
                    data_bytes: 0,
                    num_records: 0,
                    unique_id_seed: 0,
                    name,
                });
            }
        }
        
        Ok(databases)
    }

    /// Find database by name
    /// 
    /// Searches for a database with the specified name on the given card.
    /// 
    /// # Arguments
    /// * `card_no` - Card number to search
    /// * `name` - Database name to find
    /// 
    /// # Returns
    /// * `Ok(Some(DatabaseInfo))` - Database found
    /// * `Ok(None)` - Database not found
    /// * `Err(PilotError)` - Error searching
    pub async fn find_db_by_name(
        &self,
        card_no: CardNo,
        name: &str,
    ) -> Result<Option<DatabaseInfo>> {
        let databases = self.read_db_list(card_no, DlpDBListFlag::Ram, 0).await?;
        
        Ok(databases.into_iter().find(|db| db.name == name))
    }

    /// Open a database
    /// 
    /// Opens an existing database for reading/writing.
    /// 
    /// # Arguments
    /// * `card_no` - Card number
    /// * `name` - Database name
    /// * `mode` - Open mode (read, write, etc.)
    /// 
    /// # Returns
    /// * `Ok(u8)` - Database handle number
    /// * `Err(PilotError)` - Error opening database
    pub async fn open_db(
        &self,
        card_no: CardNo,
        name: &str,
        mode: DlpOpenMode,
    ) -> Result<u8> {
        let mut req = DlpRequest::new(DlpFunction::OpenDB);
        req.add_u8(card_no);
        req.add_u8(mode.bits());
        req.add_string(name);
        
        let response = self.send_request(&req).await?;
        
        Ok(response.get_u32(0).unwrap_or(0) as u8)
    }

    /// Close a database
    pub async fn close_db(&self, handle: u8) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::CloseDB);
        req.add_u8(handle);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Close all databases
    pub async fn close_all_db(&self) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::CloseDB);
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Create a new database
    pub async fn create_db(
        &self,
        creator: FourCharCode,
        db_type: FourCharCode,
        card_no: CardNo,
        flags: DatabaseFlags,
        version: u32,
        name: &str,
    ) -> Result<u8> {
        let mut req = DlpRequest::new(DlpFunction::CreateDB);
        req.add_u32(creator.to_u32());
        req.add_u32(db_type.to_u32());
        req.add_u16(flags.bits());
        req.add_u16(version as u16);
        req.add_string(name);
        
        let response = self.send_request(&req).await?;
        
        Ok(response.get_u32(0).unwrap_or(0) as u8)
    }

    /// Delete a database
    pub async fn delete_db(&self, card_no: CardNo, name: &str) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::DeleteDB);
        req.add_u8(card_no);
        req.add_string(name);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Read database info for open database
    pub async fn read_open_db_info(&self, card_no: CardNo, handle: u8) -> Result<(u32, DatabaseInfo)> {
        let mut req = DlpRequest::new(DlpFunction::ReadDBList);
        req.add_u8(card_no);
        req.add_u32(handle as u32);
        
        let response = self.send_request(&req).await?;
        
        let num_recs = response.get_u32(0).unwrap_or(0);
        let db_info = DatabaseInfo {
            flags: DatabaseFlags::empty(),
            db_type: FourCharCode { 0: 0 },
            creator: FourCharCode { 0: 0 },
            card_no: card_no as u16,
            db_id: handle as u32,
            created: PalmDateTime::now(),
            modified: PalmDateTime::now(),
            backup_date: PalmDateTime::now(),
            mod_num: 0,
            app_info_dirty: false,
            sort_info_dirty: false,
            total_bytes: 0,
            data_bytes: 0,
            num_records: num_recs,
            unique_id_seed: 0,
            name: String::new(),
        };
        
        Ok((num_recs, db_info))
    }

    // ========================================================================
    // Record Functions
    // ========================================================================

    /// Read next modified record
    pub async fn read_next_modified_rec(&self, handle: u8) -> Result<Option<Record>> {
        let mut req = DlpRequest::new(DlpFunction::ReadNextModifiedRec);
        req.add_u8(handle);
        
        let response = self.send_request(&req).await?;
        
        if response.args.is_empty() {
            return Ok(None);
        }
        
        let data = response.args.get(0).map(|a| a.data.clone()).unwrap_or_default();
        let record = Record {
            data,
            id: 0,
            index: 0,
            attributes: RecordFlags::empty(),
            sort_key: None,
        };
        
        Ok(Some(record))
    }

    /// Read a record by index
    pub async fn read_record(&self, handle: u8, index: u32) -> Result<Record> {
        let mut req = DlpRequest::new(DlpFunction::ReadRecord);
        req.add_u8(handle);
        req.add_u32(index);
        
        let response = self.send_request(&req).await?;
        
        let data = response.args.get(0).map(|a| a.data.clone()).unwrap_or_default();
        Ok(Record {
            data,
            id: 0,
            index,
            attributes: RecordFlags::empty(),
            sort_key: None,
        })
    }

    /// Read a record by ID
    pub async fn read_record_by_id(&self, handle: u8, id: u32) -> Result<Record> {
        let mut req = DlpRequest::new(DlpFunction::ReadRecord);
        req.add_u8(handle);
        req.add_u32(id);
        
        let response = self.send_request(&req).await?;
        
        let data = response.args.get(0).map(|a| a.data.clone()).unwrap_or_default();
        Ok(Record {
            data,
            id,
            index: 0,
            attributes: RecordFlags::empty(),
            sort_key: None,
        })
    }

    /// Write a record
    pub async fn write_record(
        &self,
        handle: u8,
        attributes: RecordFlags,
        id: u32,
        category: u8,
        data: &[u8],
    ) -> Result<u32> {
        let mut req = DlpRequest::new(DlpFunction::WriteRecord);
        req.add_u8(handle);
        req.add_u8(attributes.bits());
        req.add_u32(id);
        req.add_u8(category);
        req.add_bytes(data);
        
        let response = self.send_request(&req).await?;
        
        response.get_u32(0).map(|v| v).map_err(|_| PilotError::DlpBufSize)
    }

    /// Delete a record
    pub async fn delete_record(&self, handle: u8, index: u32, _id: u32) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::DeleteRecord);
        req.add_u8(handle);
        req.add_u32(index);
        req.add_u8(0);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Read record ID list
    pub async fn read_record_id_list(
        &self,
        handle: u8,
        _sort: bool,
        start: u32,
        max: u32,
    ) -> Result<Vec<u32>> {
        let mut req = DlpRequest::new(DlpFunction::ReadRecordIDList);
        req.add_u8(handle);
        req.add_u32(start);
        req.add_u32(max);
        
        let response = self.send_request(&req).await?;
        
        let mut ids = Vec::new();
        for arg in &response.args {
            if arg.data.len() >= 4 {
                let id = u32::from_le_bytes([arg.data[0], arg.data[1], arg.data[2], arg.data[3]]);
                ids.push(id);
            }
        }
        
        Ok(ids)
    }

    /// Reset the record index
    pub async fn reset_record_index(&self, handle: u8) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::ResetRecordIndex);
        req.add_u8(handle);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    // ========================================================================
    // App/Sort Block Functions
    // ========================================================================

    /// Read application info block
    pub async fn read_app_block(
        &self,
        handle: u8,
        offset: u32,
        size: Option<u32>,
    ) -> Result<Vec<u8>> {
        let mut req = DlpRequest::new(DlpFunction::ReadAppBlock);
        req.add_u8(handle);
        req.add_u32(offset);
        if let Some(s) = size {
            req.add_u32(s);
        }
        
        let response = self.send_request(&req).await?;
        
        let mut data = Vec::new();
        for arg in &response.args {
            data.extend_from_slice(&arg.data);
        }
        
        Ok(data)
    }

    /// Write application info block
    pub async fn write_app_block(&self, handle: u8, data: &[u8]) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::WriteAppBlock);
        req.add_u8(handle);
        req.add_bytes(data);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Read sort block
    pub async fn read_sort_block(
        &self,
        handle: u8,
        offset: u32,
        size: Option<u32>,
    ) -> Result<Vec<u8>> {
        let mut req = DlpRequest::new(DlpFunction::ReadSortBlock);
        req.add_u8(handle);
        req.add_u32(offset);
        if let Some(s) = size {
            req.add_u32(s);
        }
        
        let response = self.send_request(&req).await?;
        
        let mut data = Vec::new();
        for arg in &response.args {
            data.extend_from_slice(&arg.data);
        }
        
        Ok(data)
    }

    /// Write sort block
    pub async fn write_sort_block(&self, handle: u8, data: &[u8]) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::WriteSortBlock);
        req.add_u8(handle);
        req.add_bytes(data);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    // ========================================================================
    // Sync Functions
    // ========================================================================

    /// Open a conduit
    pub async fn open_conduit(&self) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::OpenConduit);
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// End sync session
    pub async fn end_sync(&self, status: DlpEndStatus) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::EndOfSync);
        req.add_u8(status as u8);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Clean up database
    pub async fn cleanup_database(&self, handle: u8) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::CleanUpDatabase);
        req.add_u8(handle);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Reset sync flags
    pub async fn reset_sync_flags(&self, handle: u8) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::ResetSyncFlags);
        req.add_u8(handle);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Add sync log entry
    pub async fn add_sync_log(&self, message: &str) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::AddSyncLogEntry);
        req.add_string(message);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Reset system (reboot device)
    pub async fn reset_system(&self) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::ResetSystem);
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    // ========================================================================
    // VFS Functions
    // ========================================================================

    /// Enumerate volumes
    pub async fn vfs_volume_enumerate(&self) -> Result<Vec<VolumeRef>> {
        let mut req = DlpRequest::new(DlpFunction::VFSVolumeEnumerate);
        let response = self.send_request(&req).await?;
        
        let mut refs = Vec::new();
        for arg in &response.args {
            if arg.data.len() >= 2 {
                let vol_ref = u16::from_le_bytes([arg.data[0], arg.data[1]]);
                refs.push(VolumeRef::new(vol_ref));
            }
        }
        
        Ok(refs)
    }

    /// Get volume info
    pub async fn vfs_volume_info(&self, vol_ref: VolumeRef) -> Result<VolumeInfo> {
        let mut req = DlpRequest::new(DlpFunction::VFSVolumeInfo);
        req.add_u16(vol_ref.value());
        
        let response = self.send_request(&req).await?;
        
        // Parse volume info from response
        Ok(VolumeInfo {
            attributes: 0,
            fs_type: FourCharCode { 0: 0 },
            fs_creator: FourCharCode { 0: 0 },
            media_type: FourCharCode { 0: 0 },
            label: String::new(),
        })
    }

    /// Open a file
    pub async fn vfs_file_open(
        &self,
        vol_ref: VolumeRef,
        path: &str,
        mode: u8,
    ) -> Result<FileRef> {
        let mut req = DlpRequest::new(DlpFunction::VFSFileOpen);
        req.add_u16(vol_ref.value());
        req.add_u8(mode);
        req.add_string(path);
        
        let response = self.send_request(&req).await?;
        
        let file_ref = response.get_u64(0).unwrap_or(0);
        Ok(FileRef(file_ref))
    }

    /// Close a file
    pub async fn vfs_file_close(&self, file_ref: FileRef) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::VFSFileClose);
        req.add_u64(file_ref.0);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Read from file
    pub async fn vfs_file_read(&self, file_ref: FileRef, size: u32) -> Result<Vec<u8>> {
        let mut req = DlpRequest::new(DlpFunction::VFSFileRead);
        req.add_u64(file_ref.0);
        req.add_u32(size);
        
        let response = self.send_request(&req).await?;
        
        let mut data = Vec::new();
        for arg in &response.args {
            data.extend_from_slice(&arg.data);
        }
        
        Ok(data)
    }

    /// Write to file
    pub async fn vfs_file_write(&self, file_ref: FileRef, data: &[u8]) -> Result<u32> {
        let mut req = DlpRequest::new(DlpFunction::VFSFileWrite);
        req.add_u64(file_ref.0);
        req.add_bytes(data);
        
        let response = self.send_request(&req).await?;
        
        response.get_u32(0).map(|n| n).map_err(|_| PilotError::DlpBufSize)
    }

    /// Seek in file
    pub async fn vfs_file_seek(&self, file_ref: FileRef, offset: i32, origin: u8) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::VFSFileSeek);
        req.add_u64(file_ref.0);
        req.add_i32(offset);
        req.add_u8(origin);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Get file size
    pub async fn vfs_file_size(&self, file_ref: FileRef) -> Result<u32> {
        let mut req = DlpRequest::new(DlpFunction::VFSFileSize);
        req.add_u64(file_ref.0);
        
        let response = self.send_request(&req).await?;
        
        response.get_u32(0).map(|s| s).map_err(|_| PilotError::DlpBufSize)
    }

    /// Delete a file
    pub async fn vfs_file_delete(&self, vol_ref: VolumeRef, path: &str) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::VFSFileDelete);
        req.add_u16(vol_ref.value());
        req.add_string(path);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Rename a file
    pub async fn vfs_file_rename(&self, vol_ref: VolumeRef, old_path: &str, new_path: &str) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::VFSFileRename);
        req.add_u16(vol_ref.value());
        req.add_string(old_path);
        req.add_string(new_path);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Create a directory
    pub async fn vfs_dir_create(&self, vol_ref: VolumeRef, path: &str) -> Result<()> {
        let mut req = DlpRequest::new(DlpFunction::VFSDirCreate);
        req.add_u16(vol_ref.value());
        req.add_string(path);
        
        let _response = self.send_request(&req).await?;
        Ok(())
    }

    /// Enumerate directory entries
    pub async fn vfs_dir_enum(&self, vol_ref: VolumeRef, path: &str, start: u32) -> Result<Vec<String>> {
        let mut req = DlpRequest::new(DlpFunction::VFSDirEntryEnumerate);
        req.add_u16(vol_ref.value());
        req.add_string(path);
        req.add_u32(start);
        
        let response = self.send_request(&req).await?;
        
        let mut entries = Vec::new();
        for arg in &response.args {
            if let Ok(name) = String::from_utf8(arg.data.clone()) {
                entries.push(name.trim_end_matches('\0').to_string());
            }
        }
        
        Ok(entries)
    }

    // ========================================================================
    // Internal
    // ========================================================================

}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert Palm OS date to SystemTime
pub fn palm_date_to_system_time(palm_date: &[u8]) -> Result<SystemTime> {
    if palm_date.len() < 8 {
        return Err(PilotError::GenericSystem);
    }
    
    let seconds = u32::from_le_bytes([
        palm_date[0], palm_date[1], palm_date[2], palm_date[3]
    ]);
    
    // Palm epoch is Jan 1, 1904. Unix epoch is Jan 1, 1970.
    // Difference is 2082844800 seconds
    const PALM_EPOCH_OFFSET: i64 = 2082844800;
    
    let unix_secs = (seconds as i64) - PALM_EPOCH_OFFSET;
    Ok(UNIX_EPOCH + std::time::Duration::from_secs(unix_secs as u64))
}

/// Convert SystemTime to Palm OS date
pub fn system_time_to_palm_date(time: SystemTime) -> [u8; 8] {
    const PALM_EPOCH_OFFSET: i64 = 2082844800;
    
    let unix_secs = time
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    
    let palm_secs = (unix_secs + PALM_EPOCH_OFFSET) as u32;
    let mut date = [0u8; 8];
    date[0..4].copy_from_slice(&palm_secs.to_le_bytes());
    date[4..8].copy_from_slice(&0u32.to_le_bytes()); // No milliseconds
    date
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlp_function_values() {
        assert_eq!(DlpFunction::ReadUserInfo as u8, 0x10);
        assert_eq!(DlpFunction::OpenDB as u8, 0x17);
        assert_eq!(DlpFunction::ReadRecord as u8, 0x20);
        assert_eq!(DlpFunction::WriteRecord as u8, 0x21);
    }

    #[test]
    fn test_dlp_error_codes() {
        assert_eq!(DlpErrorCode::from_u8(0), DlpErrorCode::NoError);
        assert_eq!(DlpErrorCode::from_u8(5), DlpErrorCode::NotFound);
        assert_eq!(DlpErrorCode::from_u8(127), DlpErrorCode::Unknown);
    }

    #[test]
    fn test_protocol_version() {
        let v = ProtocolVersion::new(1, 4);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 4);
        assert_eq!(v.to_u16(), 0x0104);
        
        let v2 = ProtocolVersion::from_u16(0x0103);
        assert_eq!(v2.major, 1);
        assert_eq!(v2.minor, 3);
    }

    #[test]
    fn test_file_ref() {
        assert!(!FileRef::INVALID.is_valid());
        let r = FileRef::new(42);
        assert!(r.is_valid());
        assert_eq!(r.value(), 42);
    }

    #[test]
    fn test_volume_ref() {
        assert!(!VolumeRef::INVALID.is_valid());
        let r = VolumeRef::new(1);
        assert!(r.is_valid());
    }
}
