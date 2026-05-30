# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `src/transport/usb.rs` — Restored `Drop` impl for USB cleanup on drop/panic
- `src/transport/usb.rs` — Restored `device_info()`, `vendor_id()`, `product_id()` helpers
- `src/transport/serial.rs` — Restored `flow_control` field to `SerialParams` with hardware flow control
- `src/protocol/dlp.rs` — 40 new DlpClient wrapper methods: Resources, Categories, Preferences, Net Sync, DB Management, Utility, VFS Volume/Metadata/File Ops, Expansion Slots, Extended Records
- `src/protocol/dlp.rs` — Public escape hatch: `DlpClient::execute()` + pub `DlpArg`/`DlpRequest`/`DlpResponse`
- `src/protocol/dlp.rs` — `SetDbInfoParams` struct replacing 9 positional parameters in `set_db_info`
- `docs/DLP_SPEC.md` — Desktop Link Protocol specification from actual implementation values
- `src/utils/strings.rs` — New `encode_palm_string()` utility (UTF-8 → CP1252) for round-trip string encoding
- `src/error.rs` — New `PilotError::SyncPoisoned` variant for mutex poisoning
- `src/transport/mod.rs` — `MockConnection::set_wouldblock_on_empty()` helper for testing async timeouts
- `src/protocol/dlp.rs` — Unit test: 20 000-byte long-format DLP arg round-trip (`test_long_format_roundtrip_20000_bytes`)
- `src/protocol/dlp.rs` — Unit test: body-read timeout with WouldBlock (`test_send_request_body_timeout`)
- `src/protocol/dlp.rs` — Unit test: `read_db_list` rejects partial/malformed responses

### Fixed
- **Critical:** `PilotSocket::serial()` — Now actually creates and stores the transport connection
- **Critical:** `PilotSocket::usb()` — Now actually creates and stores the transport connection
- **Critical:** `PilotSocket::mock()` — Restored with `MockConnection` (was deleted entirely)
- **Critical:** `src/transport/usb.rs` — Fixed USB endpoint addresses: 0x81 (IN), 0x02 (OUT)
- **Critical:** `src/transport/usb.rs` — Restored `release_interface` in `disconnect()`
- **Critical:** `src/protocol/dlp.rs` — `send_request` now reads full response body (was 4-byte header only, all wrappers returned empty data)
- `src/protocol/dlp.rs` — Fixed `read_open_db_info` using wrong function code (0x16 → 0x2B)
- `src/protocol/dlp.rs` — Fixed `DlpResponse::decode` boundary check rejecting valid empty-body responses
- `src/protocol/dlp.rs` — Fixed `decode_arg` losing arg_id for tiny-format encoded arguments
- `src/protocol/dlp.rs` — Fixed `vfs_volume_format` silently ignoring vol_ref parameter
- `src/transport/usb.rs` — Reverted `libusb` 0.2 to `libusb1-sys` 0.7 (vendored) for correct lifecycle management
- `src/transport/serial.rs` — Fixed `available_ports()` return type (was `serialport::Error` vs `std::io::Error`)
- `src/transport/mod.rs` — Restored `MockConnection`, `AsyncConnectionAdapter<T>`, `Connection for Box<T>` blanket impl
- `src/protocol/socket.rs` — Removed `#[derive(Clone)]` from `TransportConnection`, using `Option::take()` for ownership transfer
- `src/protocol/socket.rs` — Rerouted `disconnect()` and `is_connected()` through `DlpClient` after transport moved
- `src/protocol/dlp.rs` — Fixed 64KB body limit to 16MB to support DLP 1.4 extended record/resource functions
- `src/protocol/dlp.rs` — Fixed WouldBlock terminating body read (now retries instead of truncating)
- `src/protocol/dlp.rs` — Fixed `vfs_volume_info` returning hardcoded zeros (now parses real response args)
- `src/protocol/dlp.rs` — Fixed `vfs_volume_format` ignoring param argument (now sends it)
- `src/protocol/dlp.rs` — Fixed `read_next_rec_in_category` / `read_next_modified_rec_in_category` returning hardcoded id=0/index=0
- `src/protocol/dlp.rs` — Fixed `palm_date_to_system_time` panic on pre-1970 dates (returns error instead)
- `src/protocol/dlp.rs` — Removed unnecessary `mut` from `vfs_volume_enumerate` and `exp_slot_enumerate`
- `src/protocol/dlp.rs` — Added 25 unit tests covering encode/decode round-trips, all 81 function codes, error codes, arg formats, and date conversions
- `src/protocol/dlp.rs` — Fixed pre-existing DLP arg format bugs: tiny format no longer corrupts length with id bits, short/long format no longer includes id byte in data, encoded_size now matches actual encoded length, DLP_ARG_TINY_LEN corrected (0xFF→0x3F), DLP_ARG_SHORT_LEN corrected (0xFFFF→0x3FFF)
- `src/protocol/dlp.rs` — Fixed long format encode marker (0x40→0xC0) — was producing broken wire data for >16383 byte args
- `src/protocol/dlp.rs` — Fixed `read_db_list` returning hardcoded metadata (now parses flags, type, creator, dates, sizes, record count from response)
- `src/protocol/dlp.rs` — Added WouldBlock retry guard (10000 max retries) in body read loop
- `src/protocol/dlp.rs` — Fixed `encoded_size()` short format id < 0x40 guard mismatch with encode()
- `src/protocol/dlp.rs` — Fixed 7 wrapper methods returning partial hardcoded Record/DatabaseInfo metadata
- `src/protocol/dlp.rs` — Added boundary tests at DLP_ARG_TINY_LEN/SHORT_LEN transitions (63/64, 16383/16384)
- `src/protocol/dlp.rs` — `send_request` body-read loop now uses `Instant` deadline (5 s wall-clock) instead of arbitrary retry counter
- `src/protocol/dlp.rs` — `read_db_list` now validates exact 14-arg layout per DB entry and returns `InvalidData` error on truncation/mismatch
- `src/protocol/dlp.rs` — `DlpArg::encode()` long-format comment corrected (`0b01TTTTTT` → `0b11TTTTTT`)
- `src/protocol/net.rs` — Fixed `NetHandler::create_connection` panic-prone `unwrap()` (`expect("just pushed")` with safety comment)
- `src/transport/mod.rs` — `AsyncConnectionAdapter` now propagates `PilotError::SyncPoisoned` instead of panicking on poisoned `Mutex`
- `src/protocol/dlp.rs` — `DlpClient::with_transport_mut` and `send_request` now propagate `SyncPoisoned` instead of panicking on poisoned `Mutex`
- `src/records/*.rs` — All `pack()` / `pack_string()` methods now use `encode_palm_string()` (UTF-8 → CP1252) to prevent corruption of non-ASCII characters on Palm OS
- `.github/workflows/ci.yml` — Upgraded `actions/checkout@v4` → `v5` and `actions/cache@v4` → `v5` (Node.js 20 deprecation warnings)
- `src/transport/net.rs` — Fixed `InetConnection::read`/`write` violating `Read`/`Write` contract (was looping until full buffer, now returns after single partial transfer)
- `src/transport/net.rs` — Fixed `InetConnection::write` partial progress loss on `Ok(0)`
- `src/cli/db.rs` — Fixed `socket.dlp().unwrap()` panic risk (now `ok_or(DlpSocket)?`)
- `src/cli/mod.rs` — Fixed missing `#[cfg(feature = "usb")]` gate on USB fallback in `connect()`
- `src/cli/db.rs` — Fixed silent validation failure (returns `Err(InvalidArgument)` instead of `Ok(())`)
- `src/error.rs` — Changed `PilotError::FileError` from unit variant to `FileError(String)` preserving OS error context
- `src/cli/datetime.rs` — Fixed clock skew silent fallback (`unwrap_or_default()` → explicit error)
- `src/database.rs` — Fixed `DatabaseHeader.num_records` u16 truncation (changed to u32)

### Changed
- `Cargo.toml` — Restored `[features]` with `serial`/`usb` feature flags, deps made optional
- `Cargo.toml` — Added `clap` (derive), `serde_json`, `net` feature, `[[bin]] palm`
- `src/transport/net.rs` — Added `InetConnection` (TCP/IP transport for Palm HotSync): client/server modes, statistics (`rx_bytes`/`tx_bytes`/`rx_errors`/`tx_errors`), `drain_input()` (non-blocking drain), `InetState` enum
- `src/transport/net.rs` — `NetConnection`/`NetState` renamed to `InetConnection`/`InetState` to avoid collision with `protocol::net`
- `src/transport/mod.rs` — Renamed `Connection::flush` to `drain_input` to avoid `Write::flush` semantic collision
- `src/protocol/socket.rs` — Added `PilotSocket::net()` (client) and `net_listen()`/`accept()` (server) for TCP/IP HotSync
- `src/main.rs` — Full CLI with `clap` derive: `--port`, `--host`, subcommands `info`, `db` (list/info/dump/create/delete/export), `record`, `resource`, `sync`, `vfs`, `datetime`, `server`
- `src/cli/mod.rs` — Shared CLI helpers: `connect()` (auto-detects serial/network/USB), `print_table()` (aligned output)
- `src/cli/db.rs` — Database commands: list, info, dump, create, delete, export to PDB
- `src/cli/device.rs` — Device info (sys/user)
- `src/cli/datetime.rs` — Show/set device datetime
- `src/cli/record.rs` — Record list/read
- `src/cli/resource.rs` — Resource list
- `src/cli/sync.rs` — Sync command
- `src/cli/vfs.rs` — VFS volumes
- `src/cli/mod.rs` — Added `with_connection()` RAII helper ensuring disconnect always runs
- `src/cli/mod.rs` — Added 3 unit tests for `print_table()`

### Architecture
- Transport ownership: `TransportConnection` no longer requires `Clone`. Moved via `Option::take()` from `PilotSocket` into `DlpClient::new()`.
- Network transport: `InetConnection` implements full pilot-link `inet.c` semantics (client connect, server bind/listen/accept, statistics, drain_input)

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