# Known Issues — OpenPalm

> This file tracks confirmed bugs and architectural debt.
> Items are numbered for TDD workflow references.

---

## ✅ Resolved (2026-05-30)

| # | Issue | Fix | Commit |
|---|-------|-----|--------|
| 1 | DLP long-format encode marker comment (`0b01TTTTTT` → `0b11TTTTTT`) | Added 20 000-byte round-trip unit test; corrected comment | `95bd785` |
| 2 | `send_request` body-read loop lacked real timeout | Replaced retry counter with `Instant` deadline (5 s); uses `tokio::time::sleep(1 ms)` instead of `yield_now()` | `95bd785` |
| 3 | Panic-prone `unwrap()` in `NetHandler::create_connection` | Replaced with `expect("just pushed")` + safety comment; added unit test | `95bd785` |
| 4 | `read_db_list` returned hardcoded-zero metadata | Enforced exact 14-arg DLP 1.4 layout; returns `InvalidData` on truncation/mismatch | `95bd785` |
| 5 | Seven wrapper methods returned partial hardcoded metadata | Replaced `unwrap_or`/`unwrap_or_default` with strict `?` propagation in all 7 methods | `95bd785` |
| 6 | CP1252 was one-way only (decode, no encode) | Added `encode_palm_string()`; applied to all `src/records/*.rs` `pack()` methods | `95bd785` |
| 7 | MockConnection `unwrap()` on poisoned `Mutex` | Added `PilotError::SyncPoisoned`; `AsyncConnectionAdapter` and `DlpClient` now propagate instead of panic | `95bd785` |
| 8 | `decode_arg` breaking API change | Already documented in source (`index` parameter for implicit tiny-format IDs) | — |
| 9 | Node.js 20 deprecation warnings in GitHub Actions | Upgraded `actions/checkout@v4` → `v5`, `actions/cache@v4` → `v5` | `95bd785` |

---

## 🐛 Open Issues

*None at this time.*

---

## 📋 Archive

<details>
<summary>Original problem descriptions (for reference)</summary>

### 1. DLP long-format encode marker bug
- **File:** `src/protocol/dlp.rs`
- **Problem:** The `DlpArg::encode()` long-format path pushes `0xC0 | ((data_len >> 24) as u8)`. The comment above it said `0b01TTTTTT`, which implies the marker should be `0x40`, not `0xC0`. The real DLP spec uses **bits 7:6 = 11** (`0xC0`) for long format, so the code was correct and the comment was wrong. However, the discrepancy meant the implementation had never been validated against a real device for arguments larger than 16 383 bytes.
- **Impact:** High (data corruption on large writes)

### 2. `send_request` body-read loop lacked a real timeout
- **File:** `src/protocol/dlp.rs` (`DlpClient::send_request`)
- **Problem:** After reading the 4-byte header the body-read loop spun on `WouldBlock` up to `MAX_WOULDBLOCK_RETRIES = 10_000`. Between retries it called `std::thread::yield_now()` but there was **no wall-clock timeout**. On a hung transport the loop burned CPU and eventually returned `SockTimeout` after an arbitrary number of iterations rather than a predictable duration.
- **Impact:** Medium (CPU spin + unpredictable error timing)

### 3. Panic-prone `unwrap()` in `NetHandler::create_connection`
- **File:** `src/protocol/net.rs:305`
- **Problem:** `self.connections.last_mut().unwrap()` panics if the vector is empty. In the current code `push()` happens immediately before, so it cannot be empty, but this is a maintenance hazard.
- **Impact:** Low (theoretical panic on future refactoring)

### 4. `read_db_list` returned hardcoded-zero metadata
- **File:** `src/protocol/dlp.rs` (`DlpClient::read_db_list`)
- **Problem:** After parsing `name`, `flags`, `db_type`, `creator`, `card_no`, and `db_id`, the remaining fields (`created`, `modified`, `backup_date`, `mod_num`, `total_bytes`, `data_bytes`, `num_records`, `unique_id_seed`) were populated from `response.get_xxx().unwrap_or(0)`. If the mock or the real device returned fewer than 14 args per DB entry, every numeric field became `0`.
- **Impact:** Medium (sync logic relies on timestamps and sizes)

### 5. Seven wrapper methods returned partial hardcoded metadata
- **File:** `src/protocol/dlp.rs`
- **Affected methods:** `read_sys_info`, `read_storage_info`, `read_open_db_info`, `read_net_sync_info`, `exp_card_info`, `vfs_volume_size`, `vfs_custom_control`
- **Problem:** These methods ignored some response arguments and filled struct fields with `unwrap_or(0)` / `unwrap_or_default()` / hardcoded defaults. Real device data was discarded.
- **Impact:** Medium (missing info when talking to real hardware)

### 6. CP1252 was one-way only (decode, no encode)
- **File:** `src/utils/strings.rs` + all `src/records/*.rs`
- **Problem:** `decode_palm_string()` correctly converts Palm OS CP1252 bytes to Rust UTF-8 `String`. When **packing** records back to Palm format, most modules just did `text.as_bytes()` (raw UTF-8) or pushed the Rust `String` bytes directly. Any non-ASCII character (e.g. smart quotes, accented letters, Cyrillic) was written as UTF-8 multi-byte sequences, which Palm OS 3.x cannot display and may corrupt.
- **Impact:** High (international text round-trips incorrectly)

### 7. MockConnection `unwrap()` on poisoned `Mutex`
- **File:** `src/transport/mod.rs`
- **Problem:** `AsyncConnectionAdapter` used `self.inner.lock().unwrap()` in every async method. If a task panics while holding the lock, subsequent callers panic too.
- **Impact:** Low (only affects tests / async runtime stability)

### 8. `decode_arg` breaking API change
- **File:** `src/protocol/dlp.rs`
- **Problem:** `DlpResponse::decode_arg(data: &[u8], index: usize)` now requires an `index` parameter to derive implicit tiny-format IDs. This is technically a breaking change for any external code calling `decode_arg` directly, although there are no known external callers.
- **Impact:** Very low

### 9. Node.js 20 deprecation warnings in GitHub Actions
- **File:** `.github/workflows/ci.yml`
- **Problem:** `actions/checkout@v4` and `actions/cache@v4` emit Node.js 20 deprecation warnings. These are non-fatal but noisy.
- **Impact:** Low

</details>
