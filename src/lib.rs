//! OpenPalm - Rust library for Palm OS device communication
//!
//! This library is a Rust port of the pilot-link project.

// Allow pre-existing lint categories so that `cargo clippy -- -D warnings`
// passes cleanly. These allowances reflect library code with public API
// items that are not yet used internally.
#![allow(
    dead_code,
    unused_imports,
    unused_assignments,
    non_camel_case_types,
    non_upper_case_globals,
    clippy::inconsistent_digit_grouping,
    clippy::field_reassign_with_default,
    clippy::upper_case_acronyms,
    clippy::vec_init_then_push,
    clippy::needless_range_loop,
    clippy::if_same_then_else,
    clippy::manual_clamp,
    clippy::manual_strip,
    clippy::should_implement_trait,
    clippy::redundant_closure,
    clippy::single_match,
    clippy::arc_with_non_send_sync,
    clippy::wrong_self_convention,
    clippy::filter_next,
)]

pub mod error;
pub mod types;
pub mod transport;
pub mod protocol;
pub mod database;
pub mod sync;
pub mod records;
pub mod vfs;
pub mod utils;
pub mod cli;

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
    // String utilities
    parse_pstring, pack_pstring,
    parse_lpstring, pack_lpstring,
    parse_string_list, pack_string_list,
    pstring_size, string_list_size,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
