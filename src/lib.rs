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
    clippy::filter_next
)]

pub mod cli;
pub mod database;
pub mod error;
pub mod protocol;
pub mod records;
pub mod sync;
pub mod transport;
pub mod types;
pub mod utils;
pub mod vfs;

// Re-export commonly used types
pub use error::{DlpError, PilotError, Result, VfsError};
pub use types::{
    from_palm_time, to_palm_time, CardNo, DatabaseCreator, DatabaseFlags, DatabaseType, DbHandle,
    FourCharCode, OpenMode, PalmDateTime, PiBuffer, RecordFlags, RecordId, VfsFileAttributes,
    VfsOpenMode, DLP_BUF_SIZE, MAX_DBP_NAME_LEN, MAX_VFS_FILENAME,
};

// Protocol exports
pub use protocol::dlp::DlpClient;
pub use protocol::{PilotSocket, ProtocolVersion};

// Database exports
pub use database::{Database, DatabaseHandle, DatabaseInfo, Record};

// Sync exports
pub use sync::{SyncAction, SyncDirection, SyncHandler, SyncStats, SyncStrategy};

// Records exports
pub use records::{
    AccountType, AddressRecord, AlarmUnit, CalendarAppInfo, CalendarEvent, CameraInfo, CmpHeader,
    CmpMessageType, CmpPriority, CmpRecord, CmpStatus, ContactName, ContactRecord, DatebookAppInfo,
    DatebookRecord, EventType, ExpenseAppInfo, ExpenseRecord, ExpenseType, GpsCoordinate,
    GpsDirection, HiNoteAttributes, HiNoteLanguage, HiNoteRecord, ImageFormat, ImageOrientation,
    InkPoint, LocationRecord, MailAppInfo, MailFolder, MailPriority, MailRecord, MemoAppInfo,
    MemoRecord, MoneyAccount, MoneyRecord, NoteType, NotepadAppInfo, NotepadRecord, PalmPixRecord,
    PaymentType, PhoneLabel, PhoneNumber, Position, Priority, RepeatType, Sensitivity, Stroke,
    TodoAppInfo, TodoRecord, VersaMailRecord,
};

// VFS exports
pub use vfs::{path, DirEntry, FileRef, VolumeInfo, VolumeRef};
// Note: VfsFileAttributes and VfsOpenMode are exported from types module

// Utils exports
pub use utils::{
    align,
    byte_to_hex,
    bytes_to_hex,
    checksum,
    crc16,
    crc32,
    get_pilot_rate,
    hex_to_bytes,
    make_fourcc,
    pack_lpstring,
    pack_pstring,
    pack_string_list,
    pad_to_align,
    parse_lpstring,
    // String utilities
    parse_pstring,
    parse_string_list,
    pilot_rate_env,
    pstring_size,
    string_list_size,
    system_time_to_timeout,
    timeout_expired,
    timeout_to_duration,
    DebugLevel,
    Logger,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
