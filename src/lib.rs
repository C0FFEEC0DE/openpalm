//! OpenPalm - Rust library for Palm OS device communication
//!
//! This library is a Rust port of the pilot-link project.

pub mod error;
pub mod types;
pub mod transport;
pub mod protocol;
pub mod database;
pub mod sync;
pub mod records;
pub mod vfs;
pub mod utils;

// Re-export commonly used types
pub use error::{PilotError, Result, DlpError, VfsError};
pub use types::{
    PiBuffer, FourCharCode, DatabaseType, DatabaseCreator,
    RecordFlags, DatabaseFlags, OpenMode, VfsOpenMode, VfsFileAttributes,
    PalmDateTime, to_palm_time, from_palm_time,
    MAX_DBP_NAME_LEN, MAX_VFS_FILENAME, DLP_BUF_SIZE,
    RecordId, CardNo, DbHandle,
};

// Protocol exports
pub use protocol::{PilotSocket, ProtocolVersion};
pub use protocol::dlp::DlpClient;

// Database exports
pub use database::{Database, DatabaseInfo, Record, DatabaseHandle};

// Sync exports
pub use sync::{SyncHandler, SyncStrategy, SyncDirection, SyncStats, SyncAction};

// Records exports
pub use records::{
    AddressRecord, CalendarEvent, CalendarAppInfo, TodoRecord, TodoAppInfo,
    MemoRecord, MemoAppInfo, Priority, RepeatType, AlarmUnit,
    ExpenseRecord, ExpenseAppInfo, ExpenseType, PaymentType,
    NotepadRecord, NotepadAppInfo, NoteType,
    MailRecord, MailAppInfo, MailPriority, MailFolder,
    ContactRecord, ContactName, PhoneNumber, PhoneLabel,
    DatebookRecord, DatebookAppInfo, EventType,
    MoneyRecord, MoneyAccount, AccountType,
    LocationRecord, GpsCoordinate, GpsDirection, Position,
    VersaMailRecord, Sensitivity,
    HiNoteRecord, HiNoteLanguage, HiNoteAttributes, Stroke, InkPoint,
    PalmPixRecord, ImageFormat, CameraInfo, ImageOrientation,
    CmpRecord, CmpMessageType, CmpPriority, CmpStatus, CmpHeader,
};

// VFS exports
pub use vfs::{
    VolumeInfo, DirEntry, VolumeRef, FileRef,
    VfsImpl,
    path,
};
// Note: VfsFileAttributes and VfsOpenMode are exported from types module

// Utils exports
pub use utils::{
    crc16, crc32, checksum,
    bytes_to_hex, hex_to_bytes, byte_to_hex,
    align, pad_to_align,
    make_fourcc,
    timeout_to_duration, timeout_expired, system_time_to_timeout,
    get_pilot_rate, pilot_rate_env,
    DebugLevel, Logger,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
