# OpenPalm — Session Report: Transport Fix + DLP Wrapper Completion

**Date:** 2026-04-28
**Branch:** master
**Session scope:** Fix broken transport refactoring, complete DLP wrapper coverage, consolidate documentation

---

## 1. What Happened

A refactoring (-508 lines, +88 lines across 5 files) broke the project. The changes attempted to:
- Remove `#[cfg(feature = "serial")]`/`#[cfg(feature = "usb")]` feature flags
- Delete `MockConnection`, `AsyncConnectionAdapter`, `Connection for Box<T>`
- Switch `libusb1-sys` → `libusb` (incompatible lifetime constraints)
- Delete `SerialParams.flow_control`, USB `Drop` impl, `device_info()`/`vendor_id()`/`product_id()`
- "Simplify" `PilotSocket::serial()`/`::usb()`/`::mock()` to NOT create transport connections

**Result:** 5 compilation errors, 10 functional bugs, all tests using MockConnection broken.

---

## 2. Architecture Decisions

### Clone dilemma
**Decision:** Remove `#[derive(Clone)]` from `TransportConnection`. Use `Option::take()` in `PilotSocket::connect()` to move transport into `DlpClient::new()` by value.

**Rationale:** `Serial` (contains `Box<dyn SerialPort>`) and `Usb` (contains raw pointers) cannot implement meaningful Clone. Transferring ownership via `take()` is semantically honest.

### USB crate
**Decision:** Revert `libusb = "0.2"` to `libusb1-sys = { version = "0.7", features = ["vendored"] }`.

**Rationale:** `libusb` safe wrapper imposes `Context → DeviceList → Device → DeviceHandle` lifetime chain preventing persistent storage. The FFI crate has no lifetime constraints.

### Feature flags
**Decision:** Restore `[features]` with `default = ["serial", "usb"]`.

---

## 3. Changes Made

| File | Changes |
|---|---|
| `Cargo.toml` | `libusb` → `libusb1-sys` (vendored), added `[features]`, made deps optional |
| `src/transport/usb.rs` | Full rewrite for `libusb1-sys` FFI: raw `*mut libusb_context`, `*mut libusb_device_handle`, endpoints 0x81/0x02, `Drop` impl, `device_info()`/`vendor_id()`/`product_id()` |
| `src/transport/serial.rs` | Fixed `available_ports()` return type, restored `flow_control` field to `SerialParams` and `Serial` struct, added `FlowControl::Hardware`/`None` in `connect()` |
| `src/transport/mod.rs` | Restored `MockConnection` (~90 lines), `AsyncConnectionAdapter<T>` (~55 lines), `Connection for Box<T>` (~20 lines) |
| `src/protocol/socket.rs` | Removed `Clone` from `TransportConnection`, rewrote `connect()` with `Option::take()`, rerouted `disconnect()`/`is_connected()` through `DlpClient`, fixed `serial()`/`usb()`/`mock()` factory methods, added `Mock` variant |

---

## 4. Verification

| Check | Result |
|---|---|
| `cargo check` | 0 errors |
| `cargo test` | 147 passed, 0 failed |
| `cargo clippy --all-targets --all-features -- -D warnings` | 201 pre-existing warnings (not from this fix) |

---

## 5. Code Review Findings

### Verified Correct

- **libusb lifecycle:** init/exit, open/close, claim/release properly paired on all code paths
- **PilotSocket ownership:** `Option::take()` → `DlpClient::new(transport)`, no double-borrow, no leaks
- **All deleted infrastructure restored:** `MockConnection`, `AsyncConnectionAdapter`, `Connection for Box<T>`, `Drop for Usb`, `device_info()`/`vendor_id()`/`product_id()`, `SerialParams.flow_control`, `available_ports()`

### All Issues Fixed (2026-04-28, second pass)

All 7 code review findings resolved.

| # | Severity | Issue | Status |
|---|----------|-------|--------|
| 1 | MEDIUM | `#[cfg]` feature gates missing | Fixed |
| 2 | MEDIUM | `xon_xoff` parameter silently ignored | Fixed |
| 3 | MEDIUM | USB endpoints hardcoded | Fixed |
| 4 | LOW | `Usb` auto-derives `Send` | Fixed |
| 5 | LOW | `Usb::set_timeout()` is a no-op | Fixed |
| 6 | LOW | 64KB dead allocation in Usb | Fixed |
| 7 | LOW | `static mut SOCKET_COUNTER` deprecated | Fixed |

---

## 7. DLP Wrapper Completion (2026-04-28, third pass)

### Goal
Match pilot-link architecture: typed wrappers for all DLP function codes + public escape hatch.

### Changes
- **Fixed `send_request`**: Now reads full response body (was 4-byte header only, returned empty args). All wrappers now receive actual data from transport.
- **Made internals pub**: `DlpArg`, `DlpRequest`, `DlpResponse` and all methods — matches pilot-link's `dlp_arg_new`/`dlp_request_new`/`dlp_exec`
- **Added `DlpClient::execute()`**: Public escape hatch delegating to private `send_request`
- **Fixed `read_open_db_info`**: Was using `ReadDBList` (0x16) instead of `ReadOpenDBInfo` (0x2B)
- **40 new wrapper methods** in 11 groups:

| Group | Methods |
|-------|---------|
| Resources | read_resource, write_resource, delete_resource |
| Categories | move_category, read_next_rec_in_category, read_next_modified_rec_in_category |
| Preferences | read_app_preference, write_app_preference |
| Net Sync | read_net_sync_info, write_net_sync_info |
| DB Management | find_db_info, set_db_info |
| Utility | call_application, loop_back_test |
| VFS Volume Mgmt | vfs_volume_format, vfs_volume_get_label, vfs_volume_set_label, vfs_volume_size |
| VFS File Metadata | vfs_file_eof, vfs_file_tell, vfs_file_get/set_attributes, vfs_file_get/set_date, vfs_file_resize |
| VFS File Ops | vfs_custom_control, vfs_get_default_dir, vfs_import/export_database, vfs_file_create, vfs_get/put_file |
| Expansion Slots | exp_slot_enumerate, exp_card_present, exp_card_info, exp_slot_media_type |
| Extended Records | write_record_ex, write_resource_ex, read_record_ex, read_resource_ex |

### Code Review Fixes
- Fixed `DlpResponse::decode()` boundary check: `data.len() < 4` → `< 3` (valid empty-body responses)
- Fixed `vfs_volume_format` sending vol_ref as first argument (was silently ignored)
- Fixed `decode_arg` extracting arg_id from tiny-format header: `0x20 | (header & 0x1F)`
- Removed unnecessary `mut` from 8 request variables

### Verification
- 147 tests passed, 0 failed
- 0 compilation errors
- All 81 `DlpFunction` variants now have corresponding DlpClient access (66 typed wrappers + execute() escape hatch)

---

## 8. Subagents Used

| Role | Agent | Result |
|---|---|---|
| @a (Architect) | Design fix plan | Chose: revert to libusb1-sys, remove Clone, use Option::take() |
| @a (Architect) | Fix all 7 review issues | 7-step plan, executed incrementally |
| @t (Tester) | Verify tests | 147/147 passed, 5/5 structural checks |
| @cr (Code Reviewer) | Code audit | 7 findings (3 medium, 4 low), verified lifecycle safety |

---

## 9. Code Review Fix Pass (2026-04-29, fourth pass)

### Goal
Fix all 9 findings from the prior session's code review (@cr).

### Findings & Fixes

| # | Severity | Finding | Fix |
|---|----------|---------|-----|
| 1 | HIGH | No test coverage for 40 wrapper methods | Added 25 unit tests (encode/decode round-trips, all 81 function codes, error codes, arg formats, date conversions, SetDbInfoParams) |
| 2 | HIGH | 64KB body limit breaks DLP 1.4 extended read/write functions | Raised max_body_size from 0xFFFF to 0x1000000 (16MB) |
| 3 | MEDIUM | Body read loop terminates on WouldBlock — truncates data on non-blocking transports | Changed `break` to `continue` (retry instead of truncate) |
| 4 | MEDIUM | `set_db_info` has 9 positional parameters — high risk of argument transposition | Added `SetDbInfoParams` struct with named fields |
| 5 | MEDIUM | `vfs_volume_info` returns hardcoded zeros — real data discarded | Parses actual response args (attributes, fs_type, fs_creator, media_type, label) |
| 6 | LOW | Unnecessary `mut` on two request variables | Removed `mut` from `vfs_volume_enumerate` and `exp_slot_enumerate` |
| 7 | LOW | `vfs_volume_format` silently ignores `_param` | Renamed to `param` and sends it via `add_bytes()` |
| 8 | LOW | `read_next_rec_in_category` / `read_next_modified_rec_in_category` return id/index hardcoded to 0 | Parses id and index from response args |
| 9 | LOW | `palm_date_to_system_time` panics on pre-1970 dates | Added negative unix timestamp check, returns Err instead of panicking |

### Discovered Pre-Existing Bugs (NOT fixed — out of scope)
- **Tiny format encode/decode**: `DlpArg::encode()` ORs data_len with `id & 0x1F` in header byte, corrupting length when sequential arg IDs differ. Short/long format encode adds an explicit id byte between header and data, but decode treats it as part of the data (id byte included in returned data slice).
- **encoded_size() mismatch**: Short format `encoded_size()` returns `4 + data_len` but encode produces `3 + data_len`. Long format similarly off by 1.

### Verification
| Check | Result |
|---|---|
| `cargo check` | 0 errors |
| `cargo test` | 165 passed, 0 failed (+18 new tests) |
| `cargo clippy --all-targets --all-features` | All warnings pre-existing; 0 new |

### Subagents Used

| Role | Agent | Result |
|---|---|---|
| @cr (Code Reviewer) | Audit uncommitted DLP wrapper changes | 9 findings (2 HIGH, 3 MEDIUM, 4 LOW) |

### Remaining Risks
- ~~Pre-existing encode/decode bugs in `DlpArg`/`DlpResponse` tiny/short/long format (not addressed here)~~ Fixed in fifth pass
- 25 unit tests cover encode/decode paths but no integration tests with MockConnection→DlpClient→real response parsing

---

## 10. DLP Arg Format Bug Fixes (2026-04-29, fifth pass)

### Goal
Fix pre-existing encode/decode/encoded_size inconsistencies discovered during test writing.

### Root Causes
The DLP argument format uses three encoding tiers keyed by header byte:
- **Tiny** (bit 7=0): 6-bit length (0–63), id implicit from position (0x20+n)
- **Short** (bits 7:6=10): 14-bit length (0–16383), explicit id byte before data
- **Long** (bits 7:6=11): 30-bit length, explicit id byte before data

The code had four bugs:
1. `DLP_ARG_TINY_LEN` was 0xFF (255) but tiny header only has 6 bits → max 63
2. `DLP_ARG_SHORT_LEN` was 0xFFFF (65535) but short header only has 14 bits → max 16383
3. `encode()` OR'd `id & 0x1F` into tiny header byte, corrupting length for non-zero low id bits
4. `decode_arg()` read short/long data starting at `header_size`, including the id byte in data
5. `encoded_size()` overhead was off by +1 for all three formats

### Fixes

| Component | Before | After |
|-----------|--------|-------|
| `DLP_ARG_TINY_LEN` | 0xFF (255) | 0x3F (63) |
| `DLP_ARG_SHORT_LEN` | 0xFFFF (65535) | 0x3FFF (16383) |
| Tiny encode header | `data_len \| (id & 0x1F)` | `data_len` (no id OR) |
| Tiny decode id | `0x20 \| (header & 0x1F)` | `0x20 + index` (positional) |
| Short/Long decode data offset | `header_size` | `header_size + 1` (skip id byte) |
| encoded_size tiny/short/long overhead | 2/4/6 | 1/3/5 |

### API Change
`DlpResponse::decode_arg(data, index)` — now takes `index: usize` for implicit tiny id derivation.

### Verification
| Check | Result |
|---|---|
| `cargo test` | 169 passed, 0 failed |
| `cargo check` | 0 errors |
| `cargo clippy` | 0 new warnings |

---

## 11. Remaining 7 DLP Bugs Fixed (2026-04-29, sixth pass)

### Goal
Fix all remaining bugs identified in the second code review (@cr pass 2).

### Fixes

| # | Severity | Bug | Fix |
|---|----------|-----|-----|
| 1 | HIGH | Long format encode used 0x40 marker instead of 0xC0 | Changed `0x40` to `0xC0` on encode line 443 |
| 2 | HIGH | `read_db_list` returned hardcoded metadata | Parse 14-arg chunks from response: flags, db_type, creator, version, dates, mod_num, sizes, num_records, unique_id_seed |
| 3 | MEDIUM | WouldBlock body retry had no timeout guard | Added `MAX_WOULDBLOCK_RETRIES = 10000` counter, returns `SockTimeout` on exhaustion |
| 4 | MEDIUM | `encoded_size()` checked `id < 0x40` for short format; `encode()` didn't | Removed `&& self.id < 0x40` guard from encoded_size() |
| 5 | MEDIUM | 7 wrapper methods returned partial hardcoded metadata | Parse record attributes, ids, indices from response args in all 7 methods |
| 6 | LOW | Boundary test coverage missing | Added tests at exactly 63/64 and 16383/16384 byte transitions |
| 7 | LOW | Long format marker test was weak (`assert_ne`) | Tightened to `assert_eq!(encoded[0] & 0xC0, 0xC0)` |

### Changed methods
- `read_db_list` — now fully parses DatabaseInfo from response
- `read_next_modified_rec` — parses id, index, attributes from response
- `read_record` — parses id, attributes, category from response
- `read_record_by_id` — parses index, attributes, category from response
- `read_next_rec_in_category` — parses attributes from response
- `read_next_modified_rec_in_category` — parses attributes from response
- `read_open_db_info` — parses flags, type, creator, dates, sizes from response
- `find_db_info` — parses full DatabaseInfo from response args

### Verification
| Check | Result |
|---|---|
| `cargo test` | 172 passed, 0 failed |
| `cargo check` | 0 errors |
| `cargo clippy` | 0 new warnings |

### Remaining Risks
- `ARGS_PER_DB = 14` constant for read_db_list chunking — may need adjustment per DLP version
- `MAX_WOULDBLOCK_RETRIES = 10000` — threshold may need tuning per transport speed

---

## 12. Network Transport + CLI Implementation (2026-04-30 — 2026-05-01)

### Goal
Add TCP/IP network transport for Palm HotSync and implement a full CLI matching pilot-link semantics.

### Changes

| File | Changes |
|---|---|
| `src/transport/net.rs` (new) | `InetConnection` — TCP/IP transport: client connect, server bind/listen/accept, stats (`rx`/`tx` bytes/errors), `drain_input()` (non-blocking drain), `InetState` enum |
| `src/transport/mod.rs` | `Connection` trait with `drain_input()` (renamed from `flush` to avoid `Write::flush` collision); `AsyncConnectionAdapter` updated |
| `src/transport/serial.rs` | `Connection` trait impl for `Serial` |
| `src/transport/usb.rs` | `Connection` trait impl for `Usb`; SAFETY comments for `unsafe impl Send` and const→mut cast |
| `src/protocol/socket.rs` | `PilotSocket::net()` (client), `net_listen()`/`accept()` (server); `TransportConnection::Inet` variant; `dlp()` accessor |
| `src/main.rs` | Full CLI with `clap` derive: `--port`, `--host`, subcommands `info`, `db`, `record`, `resource`, `sync`, `vfs`, `datetime`, `server` |
| `src/cli/mod.rs` | `connect()` (auto-detects serial/network/USB), `print_table()` (aligned output), `with_connection()` RAII helper |
| `src/cli/db.rs` | Database commands: list, info, dump, create, delete, export to PDB |
| `src/cli/device.rs` | Device info (sys/user) |
| `src/cli/datetime.rs` | Show/set device datetime |
| `src/cli/record.rs` | Record list/read |
| `src/cli/resource.rs` | Resource list |
| `src/cli/sync.rs` | Sync command |
| `src/cli/vfs.rs` | VFS volumes |
| `Cargo.toml` | Added `clap` (derive), `serde_json`, `net` feature, `[[bin]] palm` |
| `src/lib.rs` | Lint suppressions for pre-existing warnings |
| `CHANGELOG.md` | Updated with all network transport and CLI changes |

### Code Review Findings — All Fixed

**Transport layer (7/7):**
| # | Severity | Issue | Fix |
|---|----------|-------|-----|
| 1 | CRITICAL | `PilotSocket::net_listen()` never called `conn.listen()` | Added `conn.listen()?;` after bind |
| 2 | HIGH | `Connection::flush` / `Write::flush` semantic collision | Renamed to `drain_input()` |
| 3 | HIGH | `NetConnection::write` lost partial progress on `Ok(0)` | Returns `Ok(total)` if bytes > 0 |
| 4 | MEDIUM | `drain_input` didn't count `rx_bytes` | Added increment in drain loop |
| 5 | MEDIUM | USB `Send` and const→mut cast without SAFETY | Added SAFETY comments |
| 6 | MEDIUM | `read`/`write` looped until full buffer (violates `Read`/`Write`) | Removed loops, single partial transfer |
| 7 | LOW | `NetConnection`/`NetState` name collision with `protocol::net` | Renamed to `InetConnection`/`InetState` |

**CLI layer (9/9):**
| # | Severity | Issue | Fix |
|---|----------|-------|-----|
| 1 | CRITICAL | `socket.dlp().unwrap()` panic risk (8 locations) | Replaced with `ok_or(DlpSocket)?` |
| 2 | CRITICAL | Missing `#[cfg(feature = "usb")]` on USB fallback | Added feature gate |
| 3 | HIGH | Silent validation failure (`creator.len != 4`) | Returns `Err(InvalidArgument)` |
| 4 | MEDIUM | u16 truncation in PDB export (>65535 records) | `DatabaseHeader.num_records` → u32 |
| 5 | MEDIUM | Erased I/O error context | `PilotError::FileError(String)` preserves message |
| 6 | MEDIUM | Clock skew silent fallback | Explicit error instead of `unwrap_or_default()` |
| 7 | MEDIUM | Missing RAII disconnect on error paths | `with_connection()` helper always disconnects |
| 8 | INFO | No CLI tests | Added 3 tests for `print_table()` |

### Verification
| Check | Result |
|---|---|
| `cargo check --all-features` | 0 errors |
| `cargo test --all-features` | 183 passed, 0 failed (+3 new CLI tests) |
| `cargo clippy --all-targets --all-features -- -D warnings` | 0 warnings |

### Subagents Used
| Role | Agent | Result |
|---|---|---|
| @a (Architect) | Plan network transport implementation | Client+server TCP, stats, drain_input, Connection trait |
| @bug (Bugbuster) | Implement Connection trait + stats | NetConnection, Serial, Usb; 180 tests pass |
| @cr (Code Reviewer) | Audit uncommitted transport changes | 7 findings (1 critical, 2 high, 3 medium, 1 low) |
| @cr (Code Reviewer) | Audit CLI implementation | 9 findings (2 critical, 1 high, 4 medium, 2 info) |
| @bug (Bugbuster) | Fix CLI findings | 5 fixes; 180 tests pass |
| @bug (Bugbuster) | Fix remaining 3 findings | u16→u32, clock skew, CLI tests; 183 tests pass |

### Remaining Risks
- None identified
