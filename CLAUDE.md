# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

OpenPalm is a Rust port of pilot-link — a library and CLI for communicating with Palm OS devices over HotSync. Repository: https://github.com/C0FFEEC0DE/openpalm

## Build, Test, Lint

```bash
# Build library + CLI
cargo build --release --bin op

# Run everything (unit + integration + doc tests)
cargo test

# Run a specific test
cargo test test_dlp_encode_decode --lib
cargo test test_mock_read_sys_info --test mock_integration

# Lint
cargo clippy -- -D warnings

# Features: default = ["serial", "usb"]; optional = "net"
cargo build --no-default-features --features net
```

## Protocol Stack (bottom → top)

This is the critical architecture. Data flows through the stack in both directions:

```
DLP   (Desktop Link Protocol)   — src/protocol/dlp.rs
  ↑↓
NET   (Network framing)           — src/protocol/net.rs
  ↑↓
PADP  (Packet assembly)           — src/protocol/padp.rs
  ↑↓
SLP   (Serial framing + SLIP)     — src/protocol/slp.rs
  ↑↓
Transport (Serial / USB / Mock)  — src/transport/mod.rs
```

**Entry point:** `PilotSocket` (`src/protocol/socket.rs`) wraps the full stack. Callers create a `PilotSocket`, `.connect()`, then call methods like `.read_sys_info().await` which internally use `DlpClient` to encode requests, pass them down the stack, and decode responses.

**DlpClient** (`src/protocol/dlp.rs`) is the primary API. It owns an `Arc<Mutex<TransportConnection>>` and exposes ~66 typed wrapper methods mapping to DLP function codes (0x10–0x64) plus `execute(request)` as an escape hatch. `DlpClient::send_request()` encodes a `DlpRequest`, writes it to the transport, reads the 4-byte header, then reads the body, and only then checks the error code — this ordering matters because trailing body bytes must be consumed so the next request does not read stale data.

## Palm OS Data Conventions

These invariants appear everywhere and are easy to get wrong:

- **Big-endian** for all wire-format data. Every `u16`/`u32`/`u64` in protocol packets and record data uses `to_be_bytes()` / `from_be_bytes()`.
- **Palm epoch:** January 1, 1904. Offset from Unix epoch: `2082844800` seconds. `PalmDateTime` stores seconds since Palm epoch.
- **String encoding:** Palm devices store strings in CP1252 (Windows Western), not UTF-8. Record parsers must use `utils::decode_palm_string()` (which uses `encoding_rs::WINDOWS_1252`) instead of `String::from_utf8_lossy`.
- **P-string / LP-string:** Most Palm records use null-terminated (C-style) strings. Some use length-prefixed (Pascal-style, 1-byte length). Check the specific record format before parsing.

## Record Parser Pattern

Each record type in `src/records/*.rs` follows a consistent pattern:

```rust
impl RecordType {
    pub fn parse(data: &[u8]) -> Result<Self>   // unpack from Palm binary
    pub fn pack(&self) -> Vec<u8>               // pack to Palm binary
}
```

Record parsers must validate buffer length before indexing, handle missing null terminators gracefully, and respect Palm format limits (e.g. Notepad text max 255 bytes, Todo due date uses 16-bit `YYYYYYYMMMMMDDDDD` format where `Y=year-1904` and `M=month-1`).

## Category Table Parsing

Many AppInfo blocks (Address, Datebook, Todo, Memo, etc.) share a 275-byte category structure: 16 categories × 16 bytes each + 2 bytes for last unique ID + 1 byte for flags. The helper `crate::database::parse_categories(data)` returns `(Vec<Category>, last_uniq_id, remaining_bytes)`. Prefer using this helper over manual parsing.

## Transport & Mock Testing

`src/transport/mod.rs` defines `MockConnection` for testing the full stack without hardware. It supports:

- `set_read_data(data)` — pre-load response bytes
- `set_chunk_size(n)` — limit bytes per `read()` call (simulates partial reads)
- `set_read_limit(n)` / `clear_read_limit()` — simulate packet boundaries

Integration tests live in `tests/mock_integration.rs` and exercise `PilotSocket → DlpClient → MockConnection`.

## Known Limitations

See `ISSUES.md` for the current issue tracker. As of 2026-05-30 all previously-documented DLP protocol and metadata bugs have been resolved (commits `95bd785` and `e6e24be`).

## System Dependencies

- **Fedora:** `sudo dnf install libusb1-devel pkg-config`
- **Ubuntu/Debian:** `sudo apt install libusb-1.0-0-dev pkg-config`
- **macOS:** libusb is pre-installed.
- **Windows:** Install WinUSB driver with [Zadig](https://zadig.akeo.ie/).

## License

GPL-2.0 or later.
