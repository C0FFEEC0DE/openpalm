//! Error types for openpalm library
//!
//! This module provides error handling for all layers:
//! - Protocol level errors
//! - Socket level errors
//! - DLP level errors
//! - File level errors
//! - Generic errors

use std::fmt;

/// Result type alias for openpalm operations
pub type Result<T> = std::result::Result<T, PilotError>;

/// Error codes returned by libpisock functions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PilotError {
    // Protocol level errors
    /// Aborted by other end
    ProtAborted,
    /// Can't talk with other end (incompatible protocols)
    ProtIncompatible,
    /// Bad packet (used with serial protocols)
    ProtBadPacket,

    // Socket level errors
    /// Connection has been broken
    SockDisconnected,
    /// Invalid protocol stack
    SockInvalid,
    /// Communications timeout (but link not known as broken)
    SockTimeout,
    /// Last data transfer was canceled
    SockCanceled,
    /// Generic I/O error
    SockIo,
    /// Socket can't listen/accept
    SockListener,

    // DLP level errors
    /// Provided buffer is not big enough to store data
    DlpBufSize,
    /// A non-zero error was returned by the device
    DlpPalmOs,
    /// This DLP call is not supported by the connected handheld
    DlpUnsupported,
    /// Invalid socket
    DlpSocket,
    /// Requested transfer with data block too large (>64k)
    DlpDataSize,
    /// Command error (the device returned an invalid response)
    DlpCommand,

    // File level errors
    /// Invalid prc/pdb/pqa/pi_file file
    FileInvalid,
    /// Generic error when reading/writing file
    FileError(String),
    /// File transfer was aborted by progress callback
    FileAborted,
    /// Record or resource not found
    FileNotFound,
    /// A record with same UID or resource with same type/ID already exists
    FileAlreadyExists,

    // Generic errors
    /// Not enough memory
    GenericMemory,
    /// Invalid argument(s)
    GenericArgument,
    /// Generic system error
    GenericSystem,

    // Custom errors with data
    /// DLP-specific error from device
    DlpError(u16),
    /// VFS-specific error
    VfsError(u16),
    /// Invalid data format
    InvalidData(String),
    /// Unknown character encoding
    UnknownCharEncoding,
    /// Invalid database format
    InvalidDatabase,
    /// Database not found
    DatabaseNotFound,
    /// Record not found
    RecordNotFound,
    /// Invalid argument
    InvalidArgument,
    /// Operation timed out
    Timeout,
    /// Unknown/unspecified error
    Unknown,
    /// Not implemented yet
    Unimplemented,
    /// Mutex/cell poisoned by panic in another task
    SyncPoisoned,
}

impl PilotError {
    /// Create a PilotError from a raw error code
    pub fn from_i32(code: i32) -> Self {
        match code {
            -100 => PilotError::ProtAborted,
            -101 => PilotError::ProtIncompatible,
            -102 => PilotError::ProtBadPacket,
            -200 => PilotError::SockDisconnected,
            -201 => PilotError::SockInvalid,
            -202 => PilotError::SockTimeout,
            -203 => PilotError::SockCanceled,
            -204 => PilotError::SockIo,
            -205 => PilotError::SockListener,
            -300 => PilotError::DlpBufSize,
            -301 => PilotError::DlpPalmOs,
            -302 => PilotError::DlpUnsupported,
            -303 => PilotError::DlpSocket,
            -304 => PilotError::DlpDataSize,
            -305 => PilotError::DlpCommand,
            -400 => PilotError::FileInvalid,
            -401 => PilotError::FileError(String::new()),
            -402 => PilotError::FileAborted,
            -403 => PilotError::FileNotFound,
            -404 => PilotError::FileAlreadyExists,
            -500 => PilotError::GenericMemory,
            -501 => PilotError::GenericArgument,
            -502 => PilotError::GenericSystem,
            -503 => PilotError::UnknownCharEncoding,
            -504 => PilotError::InvalidDatabase,
            -505 => PilotError::DatabaseNotFound,
            -506 => PilotError::RecordNotFound,
            -507 => PilotError::InvalidArgument,
            -508 => PilotError::Timeout,
            -509 => PilotError::SyncPoisoned,
            _ => PilotError::Unknown,
        }
    }

    /// Get a category code for this error
    pub fn category(&self) -> i32 {
        match self {
            PilotError::ProtAborted | PilotError::ProtIncompatible | PilotError::ProtBadPacket => {
                -100
            }

            PilotError::SockDisconnected
            | PilotError::SockInvalid
            | PilotError::SockTimeout
            | PilotError::SockCanceled
            | PilotError::SockIo
            | PilotError::SockListener => -200,

            PilotError::DlpBufSize
            | PilotError::DlpPalmOs
            | PilotError::DlpUnsupported
            | PilotError::DlpSocket
            | PilotError::DlpDataSize
            | PilotError::DlpCommand => -300,

            PilotError::FileInvalid
            | PilotError::FileError(_)
            | PilotError::FileAborted
            | PilotError::FileNotFound
            | PilotError::FileAlreadyExists => -400,

            PilotError::GenericMemory
            | PilotError::GenericArgument
            | PilotError::GenericSystem
            | PilotError::InvalidData(_)
            | PilotError::UnknownCharEncoding
            | PilotError::InvalidDatabase
            | PilotError::DatabaseNotFound
            | PilotError::RecordNotFound
            | PilotError::InvalidArgument
            | PilotError::Timeout
            | PilotError::SyncPoisoned => -500,

            PilotError::DlpError(_) => -301,
            PilotError::VfsError(_) => -300,
            PilotError::Unknown => -1,
            PilotError::Unimplemented => -1,
        }
    }

    /// Check if this is a protocol-level error
    pub fn is_prot_error(&self) -> bool {
        matches!(
            self,
            PilotError::ProtAborted | PilotError::ProtIncompatible | PilotError::ProtBadPacket
        )
    }

    /// Check if this is a socket-level error
    pub fn is_sock_error(&self) -> bool {
        matches!(
            self,
            PilotError::SockDisconnected
                | PilotError::SockInvalid
                | PilotError::SockTimeout
                | PilotError::SockCanceled
                | PilotError::SockIo
                | PilotError::SockListener
        )
    }

    /// Check if this is a DLP-level error
    pub fn is_dlp_error(&self) -> bool {
        matches!(
            self,
            PilotError::DlpBufSize
                | PilotError::DlpPalmOs
                | PilotError::DlpUnsupported
                | PilotError::DlpSocket
                | PilotError::DlpDataSize
                | PilotError::DlpCommand
                | PilotError::DlpError(_)
        )
    }

    /// Check if this is a file-level error
    pub fn is_file_error(&self) -> bool {
        matches!(
            self,
            PilotError::FileInvalid
                | PilotError::FileError(_)
                | PilotError::FileAborted
                | PilotError::FileNotFound
                | PilotError::FileAlreadyExists
        )
    }

    /// Check if this is a generic error
    pub fn is_generic_error(&self) -> bool {
        matches!(
            self,
            PilotError::GenericMemory | PilotError::GenericArgument | PilotError::GenericSystem
        )
    }
}

impl fmt::Display for PilotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PilotError::ProtAborted => write!(f, "Protocol: aborted by other end"),
            PilotError::ProtIncompatible => {
                write!(f, "Protocol: incompatible (can't talk with other end)")
            }
            PilotError::ProtBadPacket => write!(f, "Protocol: bad packet received"),
            PilotError::SockDisconnected => write!(f, "Socket: connection has been broken"),
            PilotError::SockInvalid => write!(f, "Socket: invalid protocol stack"),
            PilotError::SockTimeout => write!(f, "Socket: communications timeout"),
            PilotError::SockCanceled => write!(f, "Socket: transfer canceled"),
            PilotError::SockIo => write!(f, "Socket: I/O error"),
            PilotError::SockListener => write!(f, "Socket: can't listen/accept"),
            PilotError::DlpBufSize => write!(f, "DLP: buffer too small"),
            PilotError::DlpPalmOs => write!(f, "DLP: Palm OS error"),
            PilotError::DlpUnsupported => write!(f, "DLP: unsupported function"),
            PilotError::DlpSocket => write!(f, "DLP: invalid socket"),
            PilotError::DlpDataSize => write!(f, "DLP: data block too large (>64k)"),
            PilotError::DlpCommand => write!(f, "DLP: command error"),
            PilotError::DlpError(code) => write!(f, "DLP: Palm OS error code 0x{:04X}", code),
            PilotError::VfsError(code) => write!(f, "VFS: error code 0x{:04X}", code),
            PilotError::FileInvalid => write!(f, "File: invalid format"),
            PilotError::FileError(msg) => {
                if msg.is_empty() {
                    write!(f, "File: generic error")
                } else {
                    write!(f, "File: {}", msg)
                }
            }
            PilotError::FileAborted => write!(f, "File: transfer aborted"),
            PilotError::FileNotFound => write!(f, "File: not found"),
            PilotError::FileAlreadyExists => write!(f, "File: already exists"),
            PilotError::GenericMemory => write!(f, "Memory: not enough memory"),
            PilotError::GenericArgument => write!(f, "Invalid argument"),
            PilotError::GenericSystem => write!(f, "System error"),
            PilotError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            PilotError::UnknownCharEncoding => write!(f, "Unknown character encoding"),
            PilotError::InvalidDatabase => write!(f, "Invalid database format"),
            PilotError::DatabaseNotFound => write!(f, "Database not found"),
            PilotError::RecordNotFound => write!(f, "Record not found"),
            PilotError::InvalidArgument => write!(f, "Invalid argument"),
            PilotError::Timeout => write!(f, "Operation timed out"),
            PilotError::Unknown => write!(f, "Unknown error"),
            PilotError::Unimplemented => write!(f, "Not implemented"),
            PilotError::SyncPoisoned => write!(f, "Sync primitive poisoned by panic in another task"),
        }
    }
}

impl std::error::Error for PilotError {}

impl From<std::io::Error> for PilotError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::TimedOut => PilotError::SockTimeout,
            std::io::ErrorKind::ConnectionRefused => PilotError::SockDisconnected,
            std::io::ErrorKind::NotConnected => PilotError::SockDisconnected,
            std::io::ErrorKind::BrokenPipe => PilotError::SockDisconnected,
            std::io::ErrorKind::UnexpectedEof => PilotError::SockDisconnected,
            _ => PilotError::SockIo,
        }
    }
}

impl From<std::array::TryFromSliceError> for PilotError {
    fn from(_: std::array::TryFromSliceError) -> Self {
        PilotError::GenericArgument
    }
}

impl From<DlpError> for PilotError {
    fn from(err: DlpError) -> Self {
        PilotError::DlpError(err as u16)
    }
}

// DLP-specific errors (from device)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum DlpError {
    NoError = 0,
    System = 1,
    IllegalRequest = 2,
    OutOfMemory = 3,
    InvalidParameter = 4,
    NotFound = 5,
    NoneOpen = 6,
    AlreadyOpen = 7,
    TooManyOpen = 8,
    AlreadyExists = 9,
    CannotOpen = 10,
    Deleted = 11,
    Busy = 12,
    NotSupported = 13,
    Unused = 14,
    ReadOnly = 15,
    NotEnoughSpace = 16,
    LimitReached = 17,
    SyncCancelled = 18,
    WrapperError = 19,
    ArgumentMissing = 20,
    BadArgumentSize = 21,
    Unknown = 127,
}

impl DlpError {
    pub fn from_u16(code: u16) -> Self {
        match code {
            0 => DlpError::NoError,
            1 => DlpError::System,
            2 => DlpError::IllegalRequest,
            3 => DlpError::OutOfMemory,
            4 => DlpError::InvalidParameter,
            5 => DlpError::NotFound,
            6 => DlpError::NoneOpen,
            7 => DlpError::AlreadyOpen,
            8 => DlpError::TooManyOpen,
            9 => DlpError::AlreadyExists,
            10 => DlpError::CannotOpen,
            11 => DlpError::Deleted,
            12 => DlpError::Busy,
            13 => DlpError::NotSupported,
            14 => DlpError::Unused,
            15 => DlpError::ReadOnly,
            16 => DlpError::NotEnoughSpace,
            17 => DlpError::LimitReached,
            18 => DlpError::SyncCancelled,
            19 => DlpError::WrapperError,
            20 => DlpError::ArgumentMissing,
            21 => DlpError::BadArgumentSize,
            127 => DlpError::Unknown,
            _ => DlpError::Unknown,
        }
    }

    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

impl fmt::Display for DlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DlpError::NoError => write!(f, "No error"),
            DlpError::System => write!(f, "General system error"),
            DlpError::IllegalRequest => write!(f, "Illegal function"),
            DlpError::OutOfMemory => write!(f, "Out of memory"),
            DlpError::InvalidParameter => write!(f, "Invalid parameter"),
            DlpError::NotFound => write!(f, "Not found"),
            DlpError::NoneOpen => write!(f, "None open"),
            DlpError::AlreadyOpen => write!(f, "Already open"),
            DlpError::TooManyOpen => write!(f, "Too many open"),
            DlpError::AlreadyExists => write!(f, "Already exists"),
            DlpError::CannotOpen => write!(f, "Cannot open"),
            DlpError::Deleted => write!(f, "Record deleted"),
            DlpError::Busy => write!(f, "Record busy"),
            DlpError::NotSupported => write!(f, "Operation not supported"),
            DlpError::Unused => write!(f, "Unused"),
            DlpError::ReadOnly => write!(f, "Read only"),
            DlpError::NotEnoughSpace => write!(f, "Not enough space"),
            DlpError::LimitReached => write!(f, "Limit exceeded"),
            DlpError::SyncCancelled => write!(f, "Sync cancelled"),
            DlpError::WrapperError => write!(f, "Bad arg wrapper"),
            DlpError::ArgumentMissing => write!(f, "Argument missing"),
            DlpError::BadArgumentSize => write!(f, "Bad argument size"),
            DlpError::Unknown => write!(f, "Unknown error"),
        }
    }
}

// VFS errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum VfsError {
    NoError = 0,
    BufferOverflow = 1,
    GenericFileError = 2,
    InvalidFileRef = 3,
    FileStillOpen = 4,
    PermissionDenied = 5,
    FileAlreadyExists = 6,
    FileEof = 7,
    FileNotFound = 8,
    InvalidVolRef = 9,
    VolumeStillMounted = 10,
    NoFileSystem = 11,
    BadData = 12,
    NonEmptyDirectory = 13,
    InvalidPath = 14,
    VolumeFull = 15,
    Unimplemented = 16,
    NotADirectory = 17,
    IsADirectory = 18,
    DirectoryNotFound = 19,
    NameTruncated = 20,
    Unknown = 255,
}

impl VfsError {
    pub fn from_u16(code: u16) -> Self {
        match code {
            0 => VfsError::NoError,
            1 => VfsError::BufferOverflow,
            2 => VfsError::GenericFileError,
            3 => VfsError::InvalidFileRef,
            4 => VfsError::FileStillOpen,
            5 => VfsError::PermissionDenied,
            6 => VfsError::FileAlreadyExists,
            7 => VfsError::FileEof,
            8 => VfsError::FileNotFound,
            9 => VfsError::InvalidVolRef,
            10 => VfsError::VolumeStillMounted,
            11 => VfsError::NoFileSystem,
            12 => VfsError::BadData,
            13 => VfsError::NonEmptyDirectory,
            14 => VfsError::InvalidPath,
            15 => VfsError::VolumeFull,
            16 => VfsError::Unimplemented,
            17 => VfsError::NotADirectory,
            18 => VfsError::IsADirectory,
            19 => VfsError::DirectoryNotFound,
            20 => VfsError::NameTruncated,
            _ => VfsError::Unknown,
        }
    }
}

impl fmt::Display for VfsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VfsError::NoError => write!(f, "No error"),
            VfsError::BufferOverflow => write!(f, "Buffer overflow"),
            VfsError::GenericFileError => write!(f, "Generic file error"),
            VfsError::InvalidFileRef => write!(f, "Invalid file reference"),
            VfsError::FileStillOpen => write!(f, "File still open"),
            VfsError::PermissionDenied => write!(f, "Permission denied"),
            VfsError::FileAlreadyExists => write!(f, "File already exists"),
            VfsError::FileEof => write!(f, "End of file"),
            VfsError::FileNotFound => write!(f, "File not found"),
            VfsError::InvalidVolRef => write!(f, "Invalid volume reference"),
            VfsError::VolumeStillMounted => write!(f, "Volume still mounted"),
            VfsError::NoFileSystem => write!(f, "No filesystem"),
            VfsError::BadData => write!(f, "Bad data"),
            VfsError::NonEmptyDirectory => write!(f, "Non-empty directory"),
            VfsError::InvalidPath => write!(f, "Invalid path or filename"),
            VfsError::VolumeFull => write!(f, "Volume full"),
            VfsError::Unimplemented => write!(f, "Unimplemented"),
            VfsError::NotADirectory => write!(f, "Not a directory"),
            VfsError::IsADirectory => write!(f, "Is a directory"),
            VfsError::DirectoryNotFound => write!(f, "Directory not found"),
            VfsError::NameTruncated => write!(f, "Name truncated"),
            VfsError::Unknown => write!(f, "Unknown error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_i32() {
        assert!(matches!(
            PilotError::from_i32(-100),
            PilotError::ProtAborted
        ));
        assert!(matches!(
            PilotError::from_i32(-200),
            PilotError::SockDisconnected
        ));
        assert!(matches!(PilotError::from_i32(-300), PilotError::DlpBufSize));
        assert!(matches!(
            PilotError::from_i32(-400),
            PilotError::FileInvalid
        ));
        assert!(matches!(
            PilotError::from_i32(-500),
            PilotError::GenericMemory
        ));
        assert!(matches!(PilotError::from_i32(-999), PilotError::Unknown));
    }

    #[test]
    fn test_error_category_checks() {
        assert!(PilotError::ProtBadPacket.is_prot_error());
        assert!(!PilotError::ProtBadPacket.is_sock_error());

        assert!(PilotError::SockDisconnected.is_sock_error());
        assert!(!PilotError::SockDisconnected.is_dlp_error());

        assert!(PilotError::DlpBufSize.is_dlp_error());
        assert!(!PilotError::DlpBufSize.is_file_error());

        assert!(PilotError::FileNotFound.is_file_error());
        assert!(!PilotError::FileNotFound.is_generic_error());

        assert!(PilotError::GenericMemory.is_generic_error());
        assert!(!PilotError::GenericMemory.is_prot_error());
    }

    #[test]
    fn test_dlp_error_from_u16() {
        assert_eq!(DlpError::from_u16(0), DlpError::NoError);
        assert_eq!(DlpError::from_u16(5), DlpError::NotFound);
        assert_eq!(DlpError::from_u16(127), DlpError::Unknown);
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", PilotError::SockTimeout),
            "Socket: communications timeout"
        );
        assert_eq!(format!("{}", DlpError::NotFound), "Not found");
        assert_eq!(format!("{}", VfsError::FileNotFound), "File not found");
    }
}
