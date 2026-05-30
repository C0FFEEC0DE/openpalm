//! Core types for openpalm
//!
//! This module provides fundamental types used throughout the library.

pub mod buffer;
pub mod date;
pub mod flags;
pub mod fourcc;

pub use buffer::PiBuffer;
pub use date::{
    from_palm_time, to_palm_time, PalmDateTime, PALM_EPOCH_TO_UNIX_EPOCH, PALM_UNDEFINED_DATE,
};
pub use flags::{
    DatabaseFlags, OpenMode, RecordFlags, VfsFileAttributes, VfsOpenMode, VfsVolumeAttributes,
};
pub use fourcc::{DatabaseCreator, DatabaseType, FourCharCode};

/// Maximum database name length (32 characters + null terminator)
pub const MAX_DBP_NAME_LEN: usize = 34;

/// Maximum filename length in VFS
pub const MAX_VFS_FILENAME: usize = 256;

/// Default DLP buffer size (64KB - maximum record size)
pub const DLP_BUF_SIZE: usize = 0xFFFF;

/// Maximum record ID
pub const MAX_RECORD_ID: u32 = 0xFFFFFFFF;

/// Invalid record ID
pub const INVALID_RECORD_ID: u32 = 0;

/// Database record ID type (as used in Palm OS)
pub type RecordId = u32;

/// Card slot number (most devices only have one)
pub type CardNo = u8;

/// Database handle (returned by dlp_OpenDB)
pub type DbHandle = u8;
