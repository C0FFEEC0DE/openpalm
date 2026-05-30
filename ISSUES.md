# Known Issues — OpenPalm

> This file tracks confirmed bugs and architectural debt that are **not yet fixed**.
> Items are numbered for TDD workflow references.

## Protocol Layer

### 1. DLP long-format encode marker bug
- **File:** `src/protocol/dlp.rs`
- **Problem:** The `DlpArg::encode()` long-format path pushes `0xC0 | ((data_len >> 24) as u8)`.  The comment above it says `0b01TTTTTT`, which implies the marker should be `0x40`, not `0xC0`.  The real DLP spec uses **bits 7:6 = 11** (`0xC0`) for long format, so the code is probably correct and the comment is wrong.  However, the discrepancy means the implementation has never been validated against a real device for arguments larger than 16 383 bytes.  If the marker is wrong, args > 16 383 bytes will be silently truncated by the Palm side.
- **Impact:** High (data corruption on large writes)
- **Fix:** Verify against spec + pilot-link source; add round-trip unit test for a 20 000-byte argument; correct the comment or the code.

### 2. `send_request` body-read loop lacks a real timeout
- **File:** `src/protocol/dlp.rs` (`DlpClient::send_request`)
- **Problem:** After reading the 4-byte header the body-read loop spins on `WouldBlock` up to `MAX_WOULDBLOCK_RETRIES = 10_000`.  Between retries it calls `std::thread::yield_now()` but there is **no wall-clock timeout**.  On a hung transport the loop burns CPU and eventually returns `SockTimeout` after an arbitrary number of iterations rather than a predictable duration.
- **Impact:** Medium (CPU spin + unpredictable error timing)
- **Fix:** Replace the retry counter with an `Instant` deadline (e.g. 5 s).  Yield with `tokio::time::sleep` when available, or `std::thread::sleep(1ms)` in the sync path.

### 3. Panic-prone `unwrap()` in `NetHandler::create_connection`
- **File:** `src/protocol/net.rs:305`
- **Problem:** `self.connections.last_mut().unwrap()` panics if the vector is empty.  In the current code `push()` happens immediately before, so it cannot be empty, but this is a maintenance hazard.
- **Impact:** Low (theoretical panic on future refactoring)
- **Fix:** Return `Result<&mut NetConnection>` or use `expect("just pushed")` with a safety comment.

## Database / Metadata Layer

### 4. `read_db_list` returns hardcoded-zero metadata
- **File:** `src/protocol/dlp.rs` (`DlpClient::read_db_list`)
- **Problem:** After parsing `name`, `flags`, `db_type`, `creator`, `card_no`, and `db_id`, the remaining fields (`created`, `modified`, `backup_date`, `mod_num`, `total_bytes`, `data_bytes`, `num_records`, `unique_id_seed`) are populated from `response.get_xxx().unwrap_or(0)`.  If the mock or the real device returns fewer than 14 args per DB entry, every numeric field becomes `0`.
- **Impact:** Medium (sync logic relies on timestamps and sizes)
- **Fix:** Parse the exact arg layout for DLP 1.4 (14 args) and return a real error when the response is shorter than expected instead of silently defaulting to zero.

### 5. Seven wrapper methods return partial hardcoded metadata
- **File:** `src/protocol/dlp.rs`
- **Affected methods:** `read_sys_info`, `read_storage_info`, `read_open_db_info`, `read_net_sync_info`, `exp_card_info`, `vfs_volume_size`, `vfs_custom_control`
- **Problem:** These methods ignore some response arguments and fill struct fields with `unwrap_or(0)` / `unwrap_or_default()` / hardcoded defaults.  Real device data is discarded.
- **Impact:** Medium (missing info when talking to real hardware)
- **Fix:** Map every response arg to the correct struct field per DLP 1.4 spec.

## String Encoding

### 6. CP1252 is one-way only (decode, no encode)
- **File:** `src/utils/strings.rs` + all `src/records/*.rs`
- **Problem:** `decode_palm_string()` correctly converts Palm OS CP1252 bytes to Rust UTF-8 `String`.  When **packing** records back to Palm format, most modules just do `text.as_bytes()` (raw UTF-8) or push the Rust `String` bytes directly.  Any non-ASCII character (e.g. smart quotes, accented letters, Cyrillic) will be written as UTF-8 multi-byte sequences, which Palm OS 3.x cannot display and may corrupt.
- **Impact:** High (international text round-trips incorrectly)
- **Fix:** Add `encode_palm_string(src: &str) -> Result<Vec<u8>>` that converts UTF-8 → CP1252, replacing unmappable characters with `?` or `0x1A`, and use it in every `pack()` method that writes strings.

## Transport Layer

### 7. MockConnection `unwrap()` on poisoned `Mutex`
- **File:** `src/transport/mod.rs`
- **Problem:** `AsyncConnectionAdapter` uses `self.inner.lock().unwrap()` in every async method.  If a task panics while holding the lock, subsequent callers panic too.
- **Impact:** Low (only affects tests / async runtime stability)
- **Fix:** Use `lock().map_err(|_| …)?` and propagate a `PilotError::SyncPoisoned` error instead of panicking.  Same for `DlpClient::with_transport_mut` and `DlpClient::send_request`.

## Documentation / API

### 8. `decode_arg` breaking API change
- **File:** `src/protocol/dlp.rs`
- **Problem:** `DlpResponse::decode_arg(data: &[u8], index: usize)` now requires an `index` parameter to derive implicit tiny-format IDs.  This is technically a breaking change for any external code calling `decode_arg` directly, although there are no known external callers.
- **Impact:** Very low
- **Fix:** Already documented; no action required unless a public API stability guarantee is needed.

## CI / Build

### 9. Node.js 20 deprecation warnings in GitHub Actions
- **File:** `.github/workflows/ci.yml`
- **Problem:** `actions/checkout@v4` and `actions/cache@v4` emit Node.js 20 deprecation warnings.  These are non-fatal but noisy.
- **Impact:** Low
- **Fix:** Upgrade to `actions/checkout@v5` / `actions/cache@v5` when available, or pin to specific SHAs.
