# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `src/types/date.rs` - Complete `set_date()` implementation with proper Palm epoch conversion
- `src/database.rs` - Added `category` field to `Record` struct for sync support
- `src/utils/md5.rs` - Real MD5 implementation using `md5` crate (was stub)

### Fixed
- `src/types/date.rs` - Fixed `set_date()` to properly convert year/month/day to Palm timestamp
- `src/types/date.rs` - Fixed `get_date()` to work with Palm epoch instead of Unix epoch
- `src/types/date.rs` - Added day validation in `set_date()` to prevent invalid dates
- `src/sync.rs` - Fixed TODO: now extracts category from `Record.category`
- `src/utils/md5.rs` - Replaced stub with real MD5 using RFC 1321 implementation
- `src/vfs/mod.rs` - Removed redundant `VfsImpl` stubs (VFS already in `DlpClient`)
- `src/protocol/dlp.rs` - Added missing `category` field in `Record` initialization

### Changed
- `Cargo.toml` - Added `md5 = "0.7"` dependency for cryptographic MD5

### Removed
- `src/vfs/mod.rs` - Removed unused `VfsImpl` struct and 127 lines of stub code

---

## [0.1.0] - 2026-04-27

### Added

#### Core Infrastructure
- `src/error.rs` - Complete error handling with PilotError, DlpError, VfsError
- `src/types/mod.rs` - Type exports and re-exports
- `src/types/buffer.rs` - PiBuffer implementation for protocol buffers
- `src/types/date.rs` - PalmDateTime conversion utilities
- `src/types/fourcc.rs` - FourCharCode type for Palm OS identifiers
- `src/types/flags.rs` - Bitflags for records, databases, and VFS

#### Protocol Layer
- `src/protocol/dlp.rs` - Full DLP 1.4 protocol (1101 lines, 70+ functions)
- `src/protocol/slp.rs` - Serial Link Protocol implementation
- `src/protocol/padp.rs` - Palm Access Data Protocol (reliable channel)
- `src/protocol/net.rs` - Network protocol handler
- `src/protocol/socket.rs` - PilotSocket connection manager
- `src/protocol/syspkt.rs` - System packets for device info
- `src/protocol/mod.rs` - Protocol module exports

#### Transport Layer
- `src/transport/mod.rs` - Connection trait, AsyncConnection trait, MockConnection
- `src/transport/serial.rs` - Serial port communication
- `src/transport/usb.rs` - USB HotSync support

#### Database Layer
- `src/database.rs` - Database, DatabaseInfo, Record, DatabaseHandle

#### Record Types (16 implemented)
- `src/records/address.rs` - Address/Contacts database
- `src/records/calendar.rs` - Calendar events with repeat/alarm support
- `src/records/todo.rs` - To-do items with priority
- `src/records/memo.rs` - Text memos
- `src/records/expense.rs` - Expense tracking with currency
- `src/records/notepad.rs` - Quick notes
- `src/records/mail.rs` - Email messages
- `src/records/contact.rs` - Extended contact fields
- `src/records/datebook.rs` - Legacy datebook format
- `src/records/money.rs` - Financial/transaction records
- `src/records/location.rs` - GPS coordinates and Haversine distance
- `src/records/versamail.rs` - VersaMail email format
- `src/records/hinote.rs` - HiNote handwriting records
- `src/records/palmpix.rs` - PalmPix image records
- `src/records/cmp.rs` - CMP communication protocol
- `src/records/mod.rs` - Record module exports

#### VFS (Virtual File System)
- `src/vfs/mod.rs` - VFS operations, VolumeInfo, DirEntry, path utilities

#### Sync Layer
- `src/sync.rs` - SyncHandler, SyncProcessor, SyncSession, SyncStats

#### Utilities
- `src/utils/mod.rs` - Core utilities (crc16, crc32, hex, align)
- `src/utils/md5.rs` - MD5 hashing
- `src/utils/debug.rs` - Debug utilities and hex dumps
- `src/utils/sys.rs` - System information utilities

### Documentation
- `docs/ARCHITECTURE.md` - System architecture overview
- `docs/IMPLEMENTATION_PLAN.md` - Development progress tracking
- `docs/FIX_PLAN.md` - Known issues and fix tracking
- `README.md` - Project documentation with examples
- `CHANGELOG.md` - Version history

### Testing
- 145 tests across all modules (8 new tests for string utilities)
- Test coverage for all critical functions
- Integration tests with MockConnection

### Infrastructure
- `Cargo.toml` - Full dependency configuration with async support
- `.gitignore` - Git exclusions for build artifacts
- `.github/workflows/ci.yml` - GitHub Actions CI/CD pipeline

### Dependencies
- tokio (async runtime)
- serialport (serial communication)
- libusb1-sys (USB support)
- bitflags, bytes, thiserror, anyhow
- tracing, async-trait, once_cell

---

## [Unreleased]

### Planned
- VFS implementation (currently stubs)
- Async transport implementations
- Real device testing
- CLI tool
- Documentation for DLP functions

[0.1.0]: https://github.com/chaos-weaver/openpalm/releases/tag/v0.1.0