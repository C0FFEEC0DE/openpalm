# DLP Wrapper Completion Plan

> **Status: EXECUTED** — All 40 planned wrappers, escape hatch, type visibility changes, and bug fixes completed as of 2026-04-29. See `SESSION_REPORT.md` sections 7–10 for execution details.

## Current state (audit results)

- **81** `DlpFunction` variants in the enum
- **41** function codes have at least one DlpClient method referencing them
- **40** function codes have NO DlpClient method
- **44** `pub async fn` methods exist on DlpClient (some share function codes, e.g. `read_record` and `read_record_by_id` both use 0x20)

### Existing covered codes (41)

```
0x10 ReadUserInfo       0x1E WriteSortBlock        0x2F EndOfSync
0x11 WriteUserInfo      0x1F ReadNextModifiedRec   0x30 ResetRecordIndex
0x12 ReadSysInfo        0x20 ReadRecord            0x31 ReadRecordIDList
0x13 GetSysDateTime     0x21 WriteRecord           0x37 WriteNetSyncInfo
0x14 SetSysDateTime     0x22 DeleteRecord          0x38 ReadFeature
0x15 ReadStorageInfo    0x26 CleanUpDatabase       0x44 VFSFileOpen
0x16 ReadDBList         0x27 ResetSyncFlags        0x45 VFSFileClose
0x17 OpenDB             0x29 ResetSystem           0x46 VFSFileWrite
0x18 CreateDB           0x2A AddSyncLogEntry       0x47 VFSFileRead
0x19 CloseDB            0x2E OpenConduit           0x48 VFSFileDelete
0x1A DeleteDB                                   0x49 VFSFileRename
0x1B ReadAppBlock                                0x50 VFSDirCreate
0x1C WriteAppBlock                               0x51 VFSDirEntryEnumerate
0x1D ReadSortBlock                               0x55 VFSVolumeEnumerate
                                                 0x56 VFSVolumeInfo
                                                 0x5A VFSFileSeek
                                                 0x5C VFSFileSize
```

### Missing codes (40), grouped logically

```
Group A — Resources (3):
  0x23 ReadResource    0x24 WriteResource    0x25 DeleteResource

Group B — System / Utility (4):
  0x28 CallApplication    0x2B ReadOpenDBInfo*   0x3B LoopBackTest

Group C — Categories (3):
  0x2C MoveCategory    0x32 ReadNextRecInCategory    0x33 ReadNextModifiedRecInCategory

Group D — Preferences (2):
  0x34 ReadAppPreference    0x35 WriteAppPreference

Group E — Net Sync (2):
  0x36 ReadNetSyncInfo    0x37 WriteNetSyncInfo**

Group F — Database Management (3):
  0x39 FindDB    0x3A SetDBInfo    (0x2B* also fits here)

Group G — Expansion Slots (4):
  0x3C ExpSlotEnumerate    0x3D ExpCardPresent
  0x3E ExpCardInfo         0x5D ExpSlotMediaType

Group H — VFS Volume Management (4):
  0x54 VFSVolumeFormat     0x57 VFSVolumeGetLabel
  0x58 VFSVolumeSetLabel   0x59 VFSVolumeSize

Group I — VFS File Metadata (7):
  0x4A VFSFileEOF          0x4B VFSFileTell
  0x4C VFSFileGetAttributes 0x4D VFSFileSetAttributes
  0x4E VFSFileGetDate      0x4F VFSFileSetDate
  0x5B VFSFileResize

Group J — VFS File Operations (7):
  0x3F VFSCustomControl    0x40 VFSGetDefaultDir
  0x41 VFSImportDatabaseFromFile  0x42 VFSExportDatabaseToFile
  0x43 VFSFileCreate       0x52 VFSGetFile
  0x53 VFSPutFile

Group K — Extended Records (4):
  0x5E WriteRecordEx    0x5F WriteResourceEx
  0x60 ReadRecordEx     0x64 ReadResourceEx
```

`*` 0x2B ReadOpenDBInfo is a bug: the existing `read_open_db_info` method uses `DlpFunction::ReadDBList` (0x16). It must be fixed to use 0x2B.

`**` 0x37 WriteNetSyncInfo is technically "covered" by `reset_last_sync_pc`, but that method sends an empty request. A proper `write_net_sync_info` wrapper with the full argument set is needed.

---

## Design Decision 1: Escape Hatch API

### Options considered

**Option A: Make DlpRequest/DlpResponse/DlpArg fully pub**

Match pilot-link's model: `dlp_request_new` + `dlp_arg_new` + `dlp_exec(sd, &req, &res)`.

Pro: Zero new types. Exactly mirrors pilot-link.
Con: Exposes internal encoding details. DlpArg's arg ID is an implementation concern.

**Option B: Build a clean public-facing RawRequest builder**

Wrap DlpRequest in a new public type that hides encoding internals.

Pro: Cleaner public API surface.
Con: More code, diverges from pilot-link's architecture, adds abstraction layer.

**Option C: Keep DlpArg pub(crate), make DlpRequest/DlpResponse pub with all methods exposed**

A middle ground: users build requests via DlpRequest::new + add_* methods (identical to how wrappers do it today), but DlpArg remains an internal detail.

### Decision: Option A (full pub, matching pilot-link)

Rationale:
- This is a system-level protocol library. The audience is developers implementing Palm OS conduits who need exact protocol control.
- DlpRequest's `add_*` methods auto-manage arg IDs (`0x20 + self.args.len()`), so even raw users can't break the protocol.
- Making DlpArg pub costs nothing — its `new(id, data)` constructor already exists. If a user needs to construct a custom arg with a specific ID, they can.
- The existing `DlpRequest::encode()` and `DlpResponse::decode()` methods work correctly and expose no internal state that can be corrupted.
- This is the smallest possible change to achieve the goal.

### Concrete changes for escape hatch

1. **Change visibility on existing types:**
   - `struct DlpArg` → `pub struct DlpArg` (with `pub` fields: `id: u8`, `data: Vec<u8>`)
   - `struct DlpRequest` → `pub struct DlpRequest` (with `pub` field: `function: DlpFunction`)
   - `struct DlpResponse` → `pub struct DlpResponse` (with pub fields: `function: u8`, `error: DlpErrorCode`, `args: Vec<DlpArg>`)
   - All `fn new`, `fn add_*`, `fn encode`, `fn encoded_size` on DlpRequest → `pub`
   - All `fn decode`, `fn get_arg`, `fn get_u8`, `fn get_u16`, `fn get_u32`, `fn get_i32`, `fn get_u64`, `fn get_string` on DlpResponse → `pub`
   - `fn encode` on DlpArg → `pub` (for debugging/transparency)

2. **Add public passthrough method on DlpClient:**
   ```rust
   /// Send a raw DlpRequest and receive the raw DlpResponse.
   /// Escape hatch for DLP function codes that lack typed wrapper methods.
   pub async fn execute(&self, request: &DlpRequest) -> Result<DlpResponse> {
       self.send_request(request).await
   }
   ```
   Named `execute` to match pilot-link's `dlp_exec`.

3. **Keep private `send_request` unchanged.** The public escape hatch is a separate method that delegates to it. This means all existing wrappers continue working unmodified.

### Usability note

A user building a raw request for ReadResource (0x23) would write:

```rust
let mut req = DlpRequest::new(DlpFunction::ReadResource);
req.add_u8(handle);
req.add_u32(index);
let res = client.execute(&req).await?;
if res.error == DlpErrorCode::NoError {
    let data = res.get_arg(0).unwrap();
}
```

This is identical in structure to what the DlpClient wrappers do internally.

---

## Design Decision 2: Wrapper Method Organization

### Order of implementation

Each group below becomes a clearly commented section in `DlpClient`'s `impl` block. The order follows the existing convention: system functions first, then database, records, sync, VFS.

| Step | Section | Functions | Lines (est.) |
|------|---------|-----------|-------------|
| 1 | Escape hatch + type visibility | `execute` + all `pub` changes | ~30 |
| 2 | **Fix bug**: `read_open_db_info` | Switch 0x16 → 0x2B | ~3 |
| 3 | Resources | ReadResource, WriteResource, DeleteResource | ~60 |
| 4 | Categories | MoveCategory, ReadNextRecInCategory, ReadNextModifiedRecInCategory | ~75 |
| 5 | Preferences | ReadAppPreference, WriteAppPreference | ~45 |
| 6 | Net Sync | ReadNetSyncInfo, WriteNetSyncInfo (proper) | ~55 |
| 7 | DB Management | FindDB, SetDBInfo (+ fix from step 2 lives here) | ~50 |
| 8 | System | CallApplication, LoopBackTest | ~35 |
| 9 | VFS Volume Mgmt | VFSVolumeFormat, VFSVolumeGetLabel, VFSVolumeSetLabel, VFSVolumeSize | ~80 |
| 10 | VFS File Metadata | VFSFileEOF, VFSFileTell, VFSFileGetAttributes, VFSFileSetAttributes, VFSFileGetDate, VFSFileSetDate, VFSFileResize | ~120 |
| 11 | VFS File Ops | VFSCustomControl, VFSGetDefaultDir, VFSImportDatabaseFromFile, VFSExportDatabaseToFile, VFSFileCreate, VFSGetFile, VFSPutFile | ~130 |
| 12 | Expansion Slots | ExpSlotEnumerate, ExpCardPresent, ExpCardInfo, ExpSlotMediaType | ~75 |
| 13 | Extended Records | WriteRecordEx, WriteResourceEx, ReadRecordEx, ReadResourceEx | ~90 |

Total estimated new lines: ~850

### Within each section, the ordering rule

Functions that return data come before functions that write/mutate. Within each sub-group, alphabetical by operation name. This mirrors the existing convention (read before write, read before delete).

### Existing sections that need adjustment

The current `DlpClient` impl has these section headings:

```
// System Functions        (7 methods)
// Database Functions      (7 methods)
// Record Functions        (6 methods)
// App/Sort Block Functions (4 methods)
// Sync Functions          (6 methods)
// VFS Functions           (14 methods)
// Internal                (empty)
```

The new sections insert logically:

```
// System Functions        (7 existing)
// Resources               (3 new)         ← NEW SECTION
// Preferences             (2 new)         ← NEW SECTION
// Net Sync                (2 new + 1 fix) ← NEW SECTION (move reset_last_sync_pc here)
// Database Functions      (7 existing)
// Database Management     (3 new)         ← NEW SECTION (FindDB, SetDBInfo, ReadOpenDBInfo fix)
// Record Functions        (6 existing)
// Categories              (3 new)         ← NEW SECTION
// Extended Records        (4 new)         ← NEW SECTION
// App/Sort Block Functions (4 existing)
// Sync Functions          (6 existing)
// VFS Volume Management   (4 new)         ← NEW SECTION
// VFS File Operations     (7 new)         ← NEW SECTION
// VFS File Metadata       (7 new)         ← NEW SECTION
// VFS Functions           (14 existing)   (existing VFS methods stay here)
// Expansion Slots         (4 new)         ← NEW SECTION
// Utility                 (2 new)         ← NEW SECTION (CallApplication, LoopBackTest)
```

---

## Design Decision 3: Keeping the Diff Manageable

### Strategy: explicit methods, no macros

Each wrapper is written explicitly as a `pub async fn`. Rationale:

- **Each function has a unique signature**: argument count, types, and return type vary significantly. A macro that handles the union of all patterns would be more complex than 40 individual functions.
- **Doc comments are essential**: each method needs its own `///` documentation explaining what it does, its arguments, and the protocol-level semantics. These cannot be macro-generated meaningfully.
- **rust-analyzer support**: explicit methods provide IDE autocomplete, go-to-definition, and hover docs. Macro-generated methods lose all of this.
- **Discoverability**: section comments and method grouping make it easy to find related functions. Macros obscure grouping.

### Mitigation tactics for boilerplate

1. **Consistent internal pattern**: Every method follows exactly the same structure:
   ```rust
   pub async fn <name>(&self, <args>) -> Result<T> {
       let mut req = DlpRequest::new(DlpFunction::<Variant>);
       req.add_<type>(<arg>);
       // ...
       let response = self.send_request(&req).await?;
       // parse response...
       Ok(<result>)
   }
   ```
   This is ~5 lines of boilerplate per method. The unique parts are the args and response parsing.

2. **No response body parsing to fight with**: Since `send_request` discards the response body (see known issues below), initial wrappers return placeholder/default values. This keeps them short: add args, send, return Ok. Response parsing can be filled in later when the transport layer is improved.

3. **Commit granularity**: Implement in 3-4 increments, not one megacommit. Each increment adds one or two logical groups. This makes review feasible and allows partial rollout.

---

## Known issues / risks (all historical — resolved during execution)

### 1. send_request discards response body (CRITICAL) — RESOLVED

At lines 805-810, `send_request` constructs a `DlpResponse` with `args: Vec::new()`. The response body is never read from the transport. This means:

- All wrapper methods that call `response.get_u32(0)` or iterate `response.args` will always get zero/empty data.
- The wrappers appear to "work" (they compile and don't panic) but produce no actual data.
- This affects **all existing wrappers** equally, not just the new ones.

**Recommendation**: Fix `send_request` to read the full response body before adding any new wrappers. The fix involves reading the remaining bytes after the 4-byte header and passing them through `DlpResponse::decode()`. Without this fix, the 35+ new wrappers will be equally broken as the existing ones.

This fix should be step 0 in the implementation order, before the escape hatch or any new wrappers.

### 2. read_open_db_info uses wrong function code — RESOLVED

Line 1175: `DlpRequest::new(DlpFunction::ReadDBList)` should be `DlpRequest::new(DlpFunction::ReadOpenDBInfo)`. This is likely a copy-paste error. Fixing it means updating the request creation and potentially adjusting the response parsing logic.

### 3. reset_last_sync_pc sends an empty WriteNetSyncInfo request — RESOLVED (proper write_net_sync_info added)

Line 992: `DlpRequest::new(DlpFunction::WriteNetSyncInfo)` with no args. This is a partial/incorrect implementation. A proper `write_net_sync_info` wrapper needs the full argument set. The existing `reset_last_sync_pc` should either be deprecated or call through to the new proper wrapper.

### 4. DlpClient uses Arc<Mutex<TransportConnection>>

The `Arc<Mutex<>>` wrapper means `send_request` holds the lock for the entire request/response cycle. This is correct for serial/USB transports but:
- If a future transport needs concurrent requests, the Mutex becomes a bottleneck
- The `send_request` method borrows `&self`, which works because the Mutex provides interior mutability
- The public `execute` method will have the same constraint — fine for now

---

## Implementation order (execution sequence)

```
Step 0: Fix send_request to read full response body        ← prerequisite
Step 1: Make types pub + add execute() escape hatch         ← ~30 lines
Step 2: Fix read_open_db_info bug (0x16 → 0x2B)           ← ~3 lines
Step 3: Add Resources section (3 methods)                  ← ~60 lines
Step 4: Add Categories section (3 methods)                 ← ~75 lines
Step 5: Add Preferences section (2 methods)                ← ~45 lines
Step 6: Add Net Sync section (2 methods + fix)             ← ~55 lines
Step 7: Add DB Management section (3 methods)              ← ~50 lines
Step 8: Add Utility section (2 methods)                    ← ~35 lines
Step 9: Add VFS Volume Management section (4 methods)      ← ~80 lines
Step 10: Add VFS File Metadata section (7 methods)         ← ~120 lines
Step 11: Add VFS File Operations section (7 methods)       ← ~130 lines
Step 12: Add Expansion Slots section (4 methods)           ← ~75 lines
Step 13: Add Extended Records section (4 methods)          ← ~90 lines
```

Steps 1-2 are the minimum viable increment (unlocks the escape hatch). Steps 3-13 add the typed wrappers in increasing order of complexity, with the most commonly-needed groups (resources, categories, preferences) coming first.

---

## Files changed

- `/var/home/chaos_weaver/code/openpalm/src/protocol/dlp.rs` — all changes
  - Visibility changes: `DlpArg`, `DlpRequest`, `DlpResponse`, all their methods
  - New `execute()` method on DlpClient
  - Fix `read_open_db_info` function code
  - ~40 new wrapper methods with section comments
- `/var/home/chaos_weaver/code/openpalm/src/protocol/mod.rs` — may need re-exports if DlpRequest/DlpResponse/DlpArg are to be accessible through the module facade (recommended: yes, add to `pub use` list)

### Other files NOT changed (but affected transitively)

- `/var/home/chaos_weaver/code/openpalm/src/protocol/socket.rs` — `PilotSocket` delegates to DlpClient methods. If new DlpClient methods need socket-level wrappers, add them in a follow-up. The escape hatch alone is sufficient for now; socket users can call `socket.dlp().unwrap().execute(&req)`.
