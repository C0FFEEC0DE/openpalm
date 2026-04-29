# Desktop Link Protocol (DLP) Specification

OpenPalm implements DLP 1.4 — the primary protocol for Palm OS device communication over HotSync. All 81 DLP function codes have corresponding DlpClient access (66 typed wrapper methods and an `execute()` escape hatch for raw request/reply).

## Protocol Version

- **Major:** 1, **Minor:** 4 (`DLP_VERSION_MAJOR` / `DLP_VERSION_MINOR`)
- **Max record size:** 0xFFFF (65535 bytes)
- **Earlier versions:** DLP 1.0–1.2 (Palm OS 3.x–5.x), DLP 1.3 (incorrectly reports as 1.2), DLP 1.4 (Tapwave Zodiac, >64KB records), DLP 2.1 (Palm OS 6 Cobalt)

## Protocol Stack

```
DLP (Desktop Link Protocol)
  -> NET (Network framing)
    -> PADP (Reliable channel)
      -> SLP (Serial framing)
        -> Transport (Serial/USB/USB)
```

## Function Codes

### DLP 1.0 (0x10–0x31)

| Code | Name | Description |
|------|------|-------------|
| 0x10 | ReadUserInfo | Read user name, ID, last sync date |
| 0x11 | WriteUserInfo | Write user name and ID |
| 0x12 | ReadSysInfo | Read system info (ROM version, locale, DLP version) |
| 0x13 | GetSysDateTime | Get device date/time |
| 0x14 | SetSysDateTime | Set device date/time |
| 0x15 | ReadStorageInfo | Read card storage info (ROM/RAM size, free space) |
| 0x16 | ReadDBList | Enumerate databases on a card |
| 0x17 | OpenDB | Open database by name |
| 0x18 | CreateDB | Create new database |
| 0x19 | CloseDB | Close database handle |
| 0x1A | DeleteDB | Delete database by name |
| 0x1B | ReadAppBlock | Read application info block |
| 0x1C | WriteAppBlock | Write application info block |
| 0x1D | ReadSortBlock | Read sort info block |
| 0x1E | WriteSortBlock | Write sort info block |
| 0x1F | ReadNextModifiedRec | Read next modified record |
| 0x20 | ReadRecord | Read record by index |
| 0x21 | WriteRecord | Write/create record |
| 0x22 | DeleteRecord | Delete record by index |
| 0x23 | ReadResource | Read resource by index |
| 0x24 | WriteResource | Write resource |
| 0x25 | DeleteResource | Delete resource |
| 0x26 | CleanUpDatabase | Purge deleted/archived records |
| 0x27 | ResetSyncFlags | Clear dirty flags on all records |
| 0x28 | CallApplication | Call application by creator |
| 0x29 | ResetSystem | Soft reset the device |
| 0x2A | AddSyncLogEntry | Write entry to HotSync log |
| 0x2B | ReadOpenDBInfo | Get info about an open database |
| 0x2C | MoveCategory | Move a category |
| 0x2E | OpenConduit | Open sync conduit |
| 0x2F | EndOfSync | End of sync session |
| 0x30 | ResetRecordIndex | Reset next-modified-record iterator |
| 0x31 | ReadRecordIDList | List all record IDs |

### DLP 1.1 (0x32–0x38)

| Code | Name | Description |
|------|------|-------------|
| 0x32 | ReadNextRecInCategory | Read next record in category |
| 0x33 | ReadNextModifiedRecInCategory | Read next modified in category |
| 0x34 | ReadAppPreference | Read app preference |
| 0x35 | WriteAppPreference | Write app preference |
| 0x36 | ReadNetSyncInfo | Read network sync info |
| 0x37 | WriteNetSyncInfo | Write network sync info |
| 0x38 | ReadFeature | Read feature value by creator |

### DLP 1.2 (0x39–0x3A)

| Code | Name | Description |
|------|------|-------------|
| 0x39 | FindDB | Find database by name |
| 0x3A | SetDBInfo | Set database info |

### DLP 1.3 (0x3B–0x5C)

| Code | Name | Description |
|------|------|-------------|
| 0x3B | LoopBackTest | Loop-back test |
| 0x3C | ExpSlotEnumerate | Enumerate expansion slots |
| 0x3D | ExpCardPresent | Check if card present |
| 0x3E | ExpCardInfo | Get expansion card info |
| 0x3F | VFSCustomControl | Custom VFS control |
| 0x40 | VFSGetDefaultDir | Get VFS default directory |
| 0x41 | VFSImportDatabaseFromFile | Import DB from VFS file |
| 0x42 | VFSExportDatabaseToFile | Export DB to VFS file |
| 0x43 | VFSFileCreate | Create VFS file |
| 0x44 | VFSFileOpen | Open VFS file |
| 0x45 | VFSFileClose | Close VFS file |
| 0x46 | VFSFileWrite | Write to VFS file |
| 0x47 | VFSFileRead | Read from VFS file |
| 0x48 | VFSFileDelete | Delete VFS file |
| 0x49 | VFSFileRename | Rename VFS file |
| 0x4A | VFSFileEOF | VFS end-of-file check |
| 0x4B | VFSFileTell | VFS current position |
| 0x4C | VFSFileGetAttributes | Get VFS file attributes |
| 0x4D | VFSFileSetAttributes | Set VFS file attributes |
| 0x4E | VFSFileGetDate | Get VFS file date |
| 0x4F | VFSFileSetDate | Set VFS file date |
| 0x50 | VFSDirCreate | Create VFS directory |
| 0x51 | VFSDirEntryEnumerate | Enumerate VFS directory |
| 0x52 | VFSGetFile | Get file from device |
| 0x53 | VFSPutFile | Put file to device |
| 0x54 | VFSVolumeFormat | Format VFS volume |
| 0x55 | VFSVolumeEnumerate | List all mounted volumes |
| 0x56 | VFSVolumeInfo | Get volume info |
| 0x57 | VFSVolumeGetLabel | Get volume label |
| 0x58 | VFSVolumeSetLabel | Set volume label |
| 0x59 | VFSVolumeSize | Get volume size |
| 0x5A | VFSFileSeek | Seek within VFS file |
| 0x5B | VFSFileResize | Resize VFS file |
| 0x5C | VFSFileSize | Get VFS file size |

### DLP 1.4 (0x5D–0x64)

| Code | Name | Description |
|------|------|-------------|
| 0x5D | ExpSlotMediaType | Expansion slot media type |
| 0x5E | WriteRecordEx | Write record (extended, >64KB) |
| 0x5F | WriteResourceEx | Write resource (extended) |
| 0x60 | ReadRecordEx | Read record (extended) |
| 0x64 | ReadResourceEx | Read resource (extended) |

## Error Codes

| Value | Name | Description |
|-------|------|-------------|
| 0 | NoError | Success |
| 1 | System | System error |
| 2 | IllegalReq | Illegal request |
| 3 | Memory | Out of memory |
| 4 | Param | Invalid parameter |
| 5 | NotFound | Not found |
| 6 | NoneOpen | Not open |
| 7 | AlreadyOpen | Already open |
| 8 | TooManyOpen | Too many open |
| 9 | Exists | Already exists |
| 10 | Open | Cannot open |
| 11 | Deleted | Record deleted |
| 12 | Busy | Record busy |
| 13 | NotSupp | Not supported |
| 14 | Unused1 | Unused |
| 15 | ReadOnly | Read only |
| 16 | Space | Not enough space |
| 17 | Limit | Limit exceeded |
| 18 | Sync | Sync cancelled |
| 19 | Wrapper | Bad argument wrapper |
| 20 | Argument | Argument missing |
| 21 | Size | Bad argument size |
| 127 | Unknown | Unknown error |

## Open Modes

| Mode | Value | Description |
|------|-------|-------------|
| Read | 0x80 | Open for reading |
| Write | 0x40 | Open for writing |
| Exclusive | 0x20 | Exclusive access |
| Secret | 0x10 | Include secret records |
| ReadWrite | 0xC0 | Read + Write (0x80 | 0x40) |

## Database List Flags

| Flag | Value | Description |
|------|-------|-------------|
| Ram | 0x80 | List databases in RAM |
| Rom | 0x40 | List databases in ROM |
| Multiple | 0x20 | Multi-database listing |

## Sync End Status

| Status | Value | Description |
|--------|-------|-------------|
| Normal | 0 | Sync completed successfully |
| OutOfMemory | 1 | Ran out of memory |
| UserCan | 2 | Cancelled by user |
| Other | 3 | Other status |

## Data Structures

### SystemInfo
```rust
pub struct SystemInfo {
    pub rom_version:   u32,     // 0xMMmmffssbb
    pub locale:        u32,     // device locale
    pub prod_id_len:   u8,      // product ID string length
    pub prod_id:       String,  // product ID (max 32 chars)
    pub dlp_major:     u16,     // DLP major version
    pub dlp_minor:     u16,     // DLP minor version
    pub compat_major:  u16,     // compatible DLP major
    pub compat_minor:  u16,     // compatible DLP minor
    pub max_rec_size:  u32,     // max record size
}
```
Helper methods: `rom_major() -> u8`, `rom_minor() -> u8`, `rom_fix() -> u8`

### UserInfo
```rust
pub struct UserInfo {
    pub username:              String,
    pub user_id:               u32,
    pub viewer_id:             u32,
    pub last_sync_pc:          u32,
    pub last_sync_date:        Option<PalmDateTime>,
    pub successful_sync_date:  Option<PalmDateTime>,
}
```
Implements `Default`.

### StorageInfo
```rust
pub struct StorageInfo {
    pub version:        i32,
    pub rom_size:       u32,
    pub ram_size:       u32,
    pub ram_free:       u32,
    pub name:           String,
    pub manufacturer:   String,
    pub creation_date:  Option<PalmDateTime>,
}
```

### VolumeInfo
```rust
pub struct VolumeInfo {
    pub attributes:  u32,
    pub fs_type:     FourCharCode,
    pub fs_creator:  FourCharCode,
    pub media_type:  FourCharCode,
    pub label:       String,
}
```

### FileRef
```rust
pub struct FileRef(u64);

impl FileRef {
    pub const INVALID: FileRef = FileRef(0);
    pub fn new(val: u64) -> Self;
    pub fn value(&self) -> u64;
    pub fn is_valid(&self) -> bool;
}
```

### VolumeRef
```rust
pub struct VolumeRef(u16);

impl VolumeRef {
    pub const INVALID: VolumeRef = VolumeRef(0);
    pub fn new(val: u16) -> Self;
    pub fn value(&self) -> u16;
    pub fn is_valid(&self) -> bool;
}
```

### ProtocolVersion
```rust
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    pub fn new(major: u8, minor: u8) -> Self;
    pub fn current() -> Self;                  // 1.4
    pub fn from_u16(val: u16) -> Self;
    pub fn to_u16(&self) -> u16;
}
```
Implements `Default` (returns 1.4), `Display` (format: "major.minor").

### DatabaseInfo

Defined in `src/database.rs`:
```rust
pub struct DatabaseInfo {
    pub flags:            DatabaseFlags,
    pub db_type:          FourCharCode,
    pub creator:          FourCharCode,
    pub card_no:          CardNo,
    pub db_id:            u32,
    pub created:          PalmDateTime,
    pub modified:         PalmDateTime,
    pub backup_date:      PalmDateTime,
    pub mod_num:          u32,
    pub app_info_dirty:   bool,
    pub sort_info_dirty:  bool,
    pub total_bytes:      u32,
    pub data_bytes:       u32,
    pub num_records:      u32,
    pub unique_id_seed:   u32,
    pub name:             String,
}
```

### Record

Defined in `src/database.rs`:
```rust
pub struct Record {
    pub id:         RecordId,           // unique record ID
    pub index:      u32,                // index within database
    pub attributes: RecordFlags,        // Deleted, Dirty, Busy, Secret, Archived
    pub category:   u8,                 // category index
    pub data:       Vec<u8>,            // record payload
    pub sort_key:   Option<Vec<u8>>,    // optional sort key
}
```

## DlpClient API

DlpClient wraps an `Arc<Mutex<TransportConnection>>` and provides the complete DLP interface.

### Lifecycle
- `new(transport: TransportConnection) -> Self`
- `transport() -> Arc<Mutex<TransportConnection>>`

### Version
- `set_version(version: ProtocolVersion)`
- `version() -> ProtocolVersion`
- `max_record_size() -> u32`

### System
| Method | DLP Code | Return |
|--------|----------|--------|
| `read_sys_info()` | 0x12 | `Result<SystemInfo>` |
| `read_storage_info(card_no: CardNo)` | 0x15 | `Result<StorageInfo>` |
| `read_user_info()` | 0x10 | `Result<UserInfo>` |
| `write_user_info(user: &UserInfo)` | 0x11 | `Result<()>` |
| `get_sys_datetime()` | 0x13 | `Result<PalmDateTime>` |
| `set_sys_datetime(datetime: PalmDateTime)` | 0x14 | `Result<()>` |
| `reset_last_sync_pc()` | 0x37 | `Result<()>` |
| `read_feature(creator: FourCharCode, num: i32)` | 0x38 | `Result<u32>` |
| `reset_system()` | 0x29 | `Result<()>` |

### Database
| Method | DLP Code | Return |
|--------|----------|--------|
| `read_db_list(card_no, flags, start)` | 0x16 | `Result<Vec<DatabaseInfo>>` |
| `find_db_by_name(card_no, name)` | — | `Result<Option<DatabaseInfo>>` |
| `open_db(card_no, name, mode)` | 0x17 | `Result<DatabaseHandle>` |
| `close_db(handle)` | 0x19 | `Result<()>` |
| `close_all_db()` | 0x19 | `Result<()>` |
| `create_db(creator, db_type, card_no, flags, version, name)` | 0x18 | `Result<u8>` |
| `delete_db(card_no, name)` | 0x1A | `Result<()>` |
| `read_open_db_info(card_no, handle)` | 0x2B | `Result<(u32, DatabaseInfo)>` |

### Records
| Method | DLP Code | Return |
|--------|----------|--------|
| `read_record(handle, index)` | 0x20 | `Result<Record>` |
| `read_record_by_id(handle, id)` | — | `Result<Record>` |
| `write_record(handle, attributes, id, category, data)` | 0x21 | `Result<u32>` |
| `delete_record(handle, index, id)` | 0x22 | `Result<()>` |
| `read_next_modified_rec(handle)` | 0x1F | `Result<Option<Record>>` |
| `read_record_id_list(handle, sort, start, max)` | 0x31 | `Result<Vec<u32>>` |
| `reset_record_index(handle)` | 0x30 | `Result<()>` |
| `cleanup_database(handle)` | 0x26 | `Result<()>` |
| `reset_sync_flags(handle)` | 0x27 | `Result<()>` |

### Resources
| Method | DLP Code | Return |
|--------|----------|--------|
| `read_resource(handle, index)` | 0x23 | `Result<Vec<u8>>` |
| `write_resource(handle, index, data)` | 0x24 | `Result<()>` |
| `delete_resource(handle, index)` | 0x25 | `Result<()>` |

### Categories
| Method | DLP Code | Return |
|--------|----------|--------|
| `move_category(handle, src, dst)` | 0x2C | `Result<()>` |
| `read_next_rec_in_category(handle, category)` | 0x32 | `Result<Option<Record>>` |
| `read_next_modified_rec_in_category(handle, category)` | 0x33 | `Result<Option<Record>>` |

### Preferences
| Method | DLP Code | Return |
|--------|----------|--------|
| `read_app_preference(creator, id, max_size)` | 0x34 | `Result<Vec<u8>>` |
| `write_app_preference(creator, id, data)` | 0x35 | `Result<()>` |

### Net Sync
| Method | DLP Code | Return |
|--------|----------|--------|
| `read_net_sync_info()` | 0x36 | `Result<(String, String, u32)>` |
| `write_net_sync_info(host, user, pass, port)` | 0x37 | `Result<()>` |

### Database Management
| Method | DLP Code | Return |
|--------|----------|--------|
| `find_db_info(card_no, name)` | 0x39 | `Result<Option<DatabaseInfo>>` |
| `set_db_info(params: &SetDbInfoParams)` | 0x3A | `Result<()>` |

`set_db_info` accepts a `SetDbInfoParams` struct (builder-pattern, `new(handle: u8)`) instead of 9 positional parameters. Fields: `handle`, `flags`, `clear_flags`, `version`, `create_date`, `modify_date`, `backup_date`, `db_type`, `creator`.

### App/Sort Blocks
| Method | DLP Code | Return |
|--------|----------|--------|
| `read_app_block(handle, offset, size)` | 0x1B | `Result<Vec<u8>>` |
| `write_app_block(handle, data)` | 0x1C | `Result<()>` |
| `read_sort_block(handle, offset, size)` | 0x1D | `Result<Vec<u8>>` |
| `write_sort_block(handle, data)` | 0x1E | `Result<()>` |

### Utility
| Method | DLP Code | Return |
|--------|----------|--------|
| `call_application(creator, action, data)` | 0x28 | `Result<u32>` |
| `loop_back_test(data)` | 0x3B | `Result<Vec<u8>>` |

### Sync
| Method | DLP Code | Return |
|--------|----------|--------|
| `open_conduit()` | 0x2E | `Result<()>` |
| `end_sync(status: DlpEndStatus)` | 0x2F | `Result<()>` |
| `add_sync_log(message: &str)` | 0x2A | `Result<()>` |

### VFS Volume Management
| Method | DLP Code | Return |
|--------|----------|--------|
| `vfs_volume_format(vol_ref, param)` | 0x54 | `Result<()>` |
| `vfs_volume_get_label(vol_ref)` | 0x57 | `Result<String>` |
| `vfs_volume_set_label(vol_ref, label)` | 0x58 | `Result<()>` |
| `vfs_volume_size(vol_ref)` | 0x59 | `Result<(u32, u32, u32)>` |

### VFS File Operations
| Method | DLP Code | Return |
|--------|----------|--------|
| `vfs_file_open(vol_ref, path, mode)` | 0x44 | `Result<FileRef>` |
| `vfs_file_close(file_ref)` | 0x45 | `Result<()>` |
| `vfs_file_read(file_ref, size)` | 0x47 | `Result<Vec<u8>>` |
| `vfs_file_write(file_ref, data)` | 0x46 | `Result<u32>` |
| `vfs_file_seek(file_ref, offset, origin)` | 0x5A | `Result<()>` |
| `vfs_file_delete(vol_ref, path)` | 0x48 | `Result<()>` |
| `vfs_file_rename(vol_ref, old_path, new_path)` | 0x49 | `Result<()>` |
| `vfs_dir_create(vol_ref, path)` | 0x50 | `Result<()>` |
| `vfs_dir_enum(vol_ref, path, start)` | — | `Result<Vec<String>>` |
| `vfs_custom_control(op, data)` | 0x3F | `Result<Vec<u8>>` |
| `vfs_get_default_dir(vol_ref)` | 0x40 | `Result<String>` |
| `vfs_import_database_from_file(vol_ref, path)` | 0x41 | `Result<()>` |
| `vfs_export_database_to_file(handle, vol_ref, path)` | 0x42 | `Result<()>` |
| `vfs_file_create(vol_ref, path)` | 0x43 | `Result<()>` |
| `vfs_get_file(vol_ref, path, dest_path)` | 0x52 | `Result<()>` |
| `vfs_put_file(vol_ref, path, src_path)` | 0x53 | `Result<()>` |

### VFS File Metadata
| Method | DLP Code | Return |
|--------|----------|--------|
| `vfs_file_eof(file_ref)` | 0x4A | `Result<bool>` |
| `vfs_file_tell(file_ref)` | 0x4B | `Result<u32>` |
| `vfs_file_get_attributes(file_ref)` | 0x4C | `Result<u32>` |
| `vfs_file_set_attributes(file_ref, attrs)` | 0x4D | `Result<()>` |
| `vfs_file_get_date(file_ref)` | 0x4E | `Result<PalmDateTime>` |
| `vfs_file_set_date(file_ref, date)` | 0x4F | `Result<()>` |
| `vfs_file_resize(file_ref, size)` | 0x5B | `Result<()>` |
| `vfs_file_size(file_ref)` | 0x5C | `Result<u32>` |

### VFS Volume Info
| Method | DLP Code | Return |
|--------|----------|--------|
| `vfs_volume_enumerate()` | 0x55 | `Result<Vec<VolumeRef>>` |
| `vfs_volume_info(vol_ref)` | 0x56 | `Result<VolumeInfo>` |

### Expansion Slots
| Method | DLP Code | Return |
|--------|----------|--------|
| `exp_slot_enumerate()` | 0x3C | `Result<Vec<u8>>` |
| `exp_card_present(slot_ref)` | 0x3D | `Result<bool>` |
| `exp_card_info(slot_ref)` | 0x3E | `Result<Vec<u8>>` |
| `exp_slot_media_type(slot_ref)` | 0x5D | `Result<u32>` |

### Extended Records (DLP 1.4)
| Method | DLP Code | Return |
|--------|----------|--------|
| `read_record_ex(handle, index, offset, size)` | 0x60 | `Result<Vec<u8>>` |
| `read_resource_ex(handle, type, id)` | 0x64 | `Result<Vec<u8>>` |
| `write_record_ex(handle, flags, id, cat, data)` | 0x5E | `Result<u32>` |
| `write_resource_ex(handle, type, id, data)` | 0x5F | `Result<()>` |

### Escape Hatch
| Method | Return |
|--------|--------|
| `execute(request: &DlpRequest)` | `Result<DlpResponse>` |

Sends a raw `DlpRequest` and returns the raw `DlpResponse`. Covers any DLP function code not yet exposed through a typed wrapper. `DlpRequest`, `DlpResponse`, and `DlpArg` are all fully public types.

## Date Conversion

```rust
palm_date_to_system_time(palm_date: &[u8]) -> Result<SystemTime>
system_time_to_palm_date(time: SystemTime) -> [u8; 8]
```

Palm epoch: January 1, 1904. Offset from Unix epoch: 2082844800 seconds. `palm_date_to_system_time` returns `Err` for pre-1970 dates (negative Unix timestamps) instead of panicking.

## Known Issues

- **Long format encode marker:** Uses `0x40` instead of `0xC0` for the long-format header bits (bits 7:6=11). This means args larger than ~16KB are encoded with a short-format header, silently truncating the length to 14 bits. Affects only very large arguments (>16383 bytes).
- **`read_db_list` metadata:** Returns hardcoded-zero metadata for all fields except name, flags, and db_type. Record counts, sizes, timestamps, and other fields in the returned `DatabaseInfo` structs are zero.
- **Body WouldBlock retry:** The `send_request` body read loop retries on `WouldBlock` but has no timeout guard. On a permanently non-ready transport, this loops forever.
- **Seven wrapper methods** still return partial hardcoded metadata: `read_sys_info`, `read_storage_info`, `read_open_db_info`, `read_net_sync_info`, `exp_card_info`, `vfs_volume_size`, and `vfs_custom_control`. These return minimal placeholder data; the actual response bytes are discarded.
- **`decode_arg` signature change:** Now takes `index: usize` as a second parameter for tiny-format arg_id derivation (`0x20 + index`). This is a breaking API change, but `decode_arg` has zero external callers (it is used internally by `DlpResponse::decode`).
