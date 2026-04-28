# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `src/transport/usb.rs` — Restored `Drop` impl for USB cleanup on drop/panic
- `src/transport/usb.rs` — Restored `device_info()`, `vendor_id()`, `product_id()` helpers
- `src/transport/serial.rs` — Restored `flow_control` field to `SerialParams` with hardware flow control

### Fixed
- **Critical:** `PilotSocket::serial()` — Now actually creates and stores the transport connection
- **Critical:** `PilotSocket::usb()` — Now actually creates and stores the transport connection
- **Critical:** `PilotSocket::mock()` — Restored with `MockConnection` (was deleted entirely)
- **Critical:** `src/transport/usb.rs` — Fixed USB endpoint addresses: 0x81 (IN), 0x02 (OUT)
- **Critical:** `src/transport/usb.rs` — Restored `release_interface` in `disconnect()`
- `src/transport/usb.rs` — Reverted `libusb` 0.2 to `libusb1-sys` 0.7 (vendored) for correct lifecycle management
- `src/transport/serial.rs` — Fixed `available_ports()` return type (was `serialport::Error` vs `std::io::Error`)
- `src/transport/mod.rs` — Restored `MockConnection`, `AsyncConnectionAdapter<T>`, `Connection for Box<T>` blanket impl
- `src/protocol/socket.rs` — Removed `#[derive(Clone)]` from `TransportConnection`, using `Option::take()` for ownership transfer
- `src/protocol/socket.rs` — Rerouted `disconnect()` and `is_connected()` through `DlpClient` after transport moved

### Changed
- `Cargo.toml` — Restored `[features]` with `serial`/`usb` feature flags, deps made optional

### Architecture
- Transport ownership: `TransportConnection` no longer requires `Clone`. Moved via `Option::take()` from `PilotSocket` into `DlpClient::new()`.

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

[0.1.0]: https://github.com/chaos-weaver/openpalm/releases/tag/v0.1.0