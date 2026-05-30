//! Record types for openpalm
//!
//! This module provides specific record types for Palm OS databases.

mod address;
mod calendar;
mod cmp;
mod contact;
mod datebook;
mod expense;
mod hinote;
mod location;
mod mail;
mod memo;
mod money;
mod notepad;
mod palmpix;
mod todo;
mod versamail;

pub use address::AddressRecord;
pub use calendar::{AlarmUnit, CalendarAppInfo, CalendarEvent, RepeatType};
pub use cmp::{CmpHeader, CmpMessageType, CmpPriority, CmpRecord, CmpSession, CmpStatus};
pub use contact::{ContactName, ContactRecord, ImAddress, PhoneLabel, PhoneNumber, PostalAddress};
pub use datebook::{AlarmInfo, DatebookAppInfo, DatebookRecord, EventType, RepeatInfo};
pub use expense::{CurrencyInfo, ExpenseAppInfo, ExpenseRecord, ExpenseType, PaymentType};
pub use hinote::{
    HiNoteAttributes, HiNoteLanguage, HiNoteRecord, InkPoint, RecognitionMode, Stroke,
};
pub use location::{GpsCoordinate, GpsDirection, LocationRecord, Position};
pub use mail::{MailAppInfo, MailFolder, MailPriority, MailRecord};
pub use memo::{MemoAppInfo, MemoRecord};
pub use money::{AccountType, MoneyAccount, MoneyAppInfo, MoneyRecord};
pub use notepad::{NoteType, NotepadAppInfo, NotepadAttributes, NotepadRecord};
pub use palmpix::{
    CameraInfo, ImageFormat, ImageOrientation, PalmPixAttributes, PalmPixRecord, Thumbnail,
};
pub use todo::{Priority, TodoAppInfo, TodoRecord};
pub use versamail::{Attachment, Sensitivity, VersaMailAccount, VersaMailRecord};
