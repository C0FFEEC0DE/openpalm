//! Record types for openpalm
//!
//! This module provides specific record types for Palm OS databases.

mod address;
mod calendar;
mod todo;
mod memo;
mod expense;
mod notepad;
mod mail;
mod contact;
mod datebook;
mod money;
mod location;
mod versamail;
mod hinote;
mod palmpix;
mod cmp;

pub use address::AddressRecord;
pub use calendar::{CalendarEvent, CalendarAppInfo, RepeatType, AlarmUnit};
pub use todo::{TodoRecord, TodoAppInfo, Priority};
pub use memo::{MemoRecord, MemoAppInfo};
pub use expense::{ExpenseRecord, ExpenseAppInfo, ExpenseType, PaymentType, CurrencyInfo};
pub use notepad::{NotepadRecord, NotepadAppInfo, NotepadAttributes, NoteType};
pub use mail::{MailRecord, MailAppInfo, MailPriority, MailFolder};
pub use contact::{ContactRecord, ContactName, PhoneNumber, PhoneLabel, PostalAddress, ImAddress};
pub use datebook::{DatebookRecord, DatebookAppInfo, EventType, RepeatInfo, AlarmInfo};
pub use money::{MoneyRecord, MoneyAccount, MoneyAppInfo, AccountType};
pub use location::{LocationRecord, GpsCoordinate, GpsDirection, Position};
pub use versamail::{VersaMailRecord, VersaMailAccount, Attachment, Sensitivity};
pub use hinote::{HiNoteRecord, HiNoteLanguage, HiNoteAttributes, Stroke, InkPoint, RecognitionMode};
pub use palmpix::{PalmPixRecord, ImageFormat, PalmPixAttributes, CameraInfo, Thumbnail, ImageOrientation};
pub use cmp::{CmpRecord, CmpMessageType, CmpPriority, CmpStatus, CmpHeader, CmpSession};
