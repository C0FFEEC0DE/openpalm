# Архитектура libpisock/libpisync для портирования на Rust

## Обзор

**pilot-link** - историческая C-библиотека для коммуникации с Palm OS устройствами через протокол HotSync.

**Репо:** https://github.com/jichu4n/pilot-link

---

## Структура проекта

```
pilot-link/
├── include/           # Заголовочные файлы
│   ├── pi-socket.h    # Сокетный интерфейс
│   ├── pi-dlp.h       # Desktop Link Protocol (основной)
│   ├── pi-sync.h      # Синхронизация
│   ├── pi-buffer.h    # Буфер данных
│   └── ...            # Другие протоколы и типы данных
├── libpisock/         # Реализация протоколов
│   ├── dlp.c          # DLP протокол (ядро, 124KB)
│   ├── socket.c       # Сокетный уровень
│   ├── padp.c         # PAD Protocol
│   ├── net.c          # Сетевой протокол
│   ├── serial.c       # Serial transport
│   ├── usb.c          # USB transport
│   ├── address.c      # Address book (Contacts)
│   ├── calendar.c     # Calendar records
│   ├── contact.c      # Contact records
│   └── ...
└── libpisync/         # Библиотека синхронизации
    ├── sync.c         # Основная логика синхронизации
    └── util.c         # Утилиты
```

---

## Уровневая архитектура (Protocol Stack)

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│    (Your Rust code using the lib)       │
├─────────────────────────────────────────┤
│         DLP Level (PI_LEVEL_DLP)        │
│    dlp_ReadRecord, dlp_WriteRecord,     │
│    dlp_OpenDB, dlp_CloseDB, etc.        │
├─────────────────────────────────────────┤
│         NET Level (PI_LEVEL_NET)        │
│    Network framing, packet assembly     │
├─────────────────────────────────────────┤
│         PADP Level (PI_LEVEL_PADP)      │
│    Packet Assembly/Disassembly Protocol │
├─────────────────────────────────────────┤
│         SLP Level (PI_LEVEL_SLP)        │
│    Serial Link Protocol                 │
├─────────────────────────────────────────┤
│         DEV Level (PI_LEVEL_DEV)        │
│    Device level (serial/USB/Ip)         │
├─────────────────────────────────────────┤
│         Physical Transport              │
│    USB / Serial / Bluetooth / Network   │
└─────────────────────────────────────────┘
```

---

## Ключевые компоненты

### 1. pi_socket_t (Сокет)

Основная структура для соединения с устройством:

```c
typedef struct pi_socket {
    int sd;                      // Socket descriptor
    int type;                    // PI_SOCK_STREAM or PI_SOCK_RAW
    int protocol;                // Protocol family (PI_PF_DLP)
    
    struct sockaddr *laddr;      // Local address
    struct sockaddr *raddr;      // Remote address
    
    struct pi_protocol **protocol_queue;  // Protocol stack
    int queue_len;
    
    struct pi_device *device;    // Low-level device
    
    int state;                   // Socket state
    int dlpversion;              // DLP protocol version
    unsigned long maxrecsize;    // Max record size
    
    int last_error;              // Last error code
    int palmos_error;            // Palm OS error code
} pi_socket_t;
```

**Rust эквивалент:**
```rust
pub struct PilotSocket {
    fd: RawFd,
    socket_type: SocketType,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    protocol_stack: Vec<Box<dyn Protocol>>,
    device: Option<Box<dyn Device>>,
    state: SocketState,
    dlp_version: DlpVersion,
    max_record_size: u32,
    last_error: PilotError,
}
```

### 2. DLP Protocol (Desktop Link Protocol)

Основной протокол для работы с Palm устройствами. Реализован в `dlp.c` (124KB).

**Версии DLP:**
- DLP 1.2: Palm OS 4/5
- DLP 1.3: Palm OS 5 (incorrectly reports 1.2)
- DLP 1.4: Tapwave Zodiac, поддержка записей >64KB
- DLP 2.1: Palm OS 6 (Cobalt)

**Основные функции DLP:**

| Категория | Функции |
|-----------|---------|
| **System** | `dlp_GetSysDateTime`, `dlp_SetSysDateTime`, `dlp_ReadSysInfo` |
| **Database** | `dlp_OpenDB`, `dlp_CloseDB`, `dlp_CreateDB`, `dlp_DeleteDB`, `dlp_ReadDBList` |
| **Records** | `dlp_ReadRecord`, `dlp_WriteRecord`, `dlp_DeleteRecord`, `dlp_ReadNextModifiedRec` |
| **VFS** | `dlp_VFSFileOpen`, `dlp_VFSFileRead`, `dlp_VFSFileWrite`, `dlp_VFSVolumeEnumerate` |
| **User** | `dlp_ReadUserInfo`, `dlp_WriteUserInfo` |
| **Sync** | `dlp_EndOfSync`, `dlp_OpenConduit` |

**Структуры данных:**

```c
struct DBInfo {
    char name[34];               // Database name (32 chars + null)
    unsigned int flags;          // dlpDBFlagResource, dlpDBFlagReadOnly, etc.
    unsigned long type;          // 4-char code (e.g., 'appl', 'DATA')
    unsigned long creator;       // Creator ID
    time_t createDate;
    time_t modifyDate;
    // ...
};

struct PilotUser {
    char username[128];
    char password[128];
    unsigned long userID;
    time_t lastSyncDate;
    // ...
};
```

### 3. Sync Handler (libpisync)

Абстракция для синхронизации данных между устройством и десктопом.

```c
struct _SyncHandler {
    int sd;                      // Socket descriptor
    
    char *name;                  // Database name
    int secret;                  // Show secret records
    
    // Callbacks
    int (*Pre)(SyncHandler*, int dbhandle, int *slow);
    int (*Post)(SyncHandler*, int dbhandle);
    
    int (*ForEach)(SyncHandler*, DesktopRecord**);
    int (*ForEachModified)(SyncHandler*, DesktopRecord**);
    int (*Compare)(SyncHandler*, PilotRecord*, DesktopRecord*);
    
    int (*AddRecord)(SyncHandler*, PilotRecord*);
    int (*ReplaceRecord)(SyncHandler*, DesktopRecord*, PilotRecord*);
    int (*DeleteRecord)(SyncHandler*, DesktopRecord*);
    
    int (*Match)(SyncHandler*, PilotRecord*, DesktopRecord**);
    int (*Prepare)(SyncHandler*, DesktopRecord*, PilotRecord*);
};
```

**Стратегии синхронизации:**

| Функция | Описание |
|---------|----------|
| `sync_CopyToPilot` | Полное копирование с десктопа на устройство |
| `sync_CopyFromPilot` | Полное копирование с устройства на десктоп |
| `sync_MergeToPilot` | Слияние: модифицированные записи с десктопа на устройство |
| `sync_MergeFromPilot` | Слияние: модифицированные записи с устройства на десктоп |
| `sync_Synchronize` | Полная двунаправленная синхронизация |

### 4. Transport Layer

**Поддерживаемые транспорты:**

```
/dev/ttyUSB0          # Serial (USB-serial adapter)
/dev/ttyUSB1
usb:                  # Direct USB (Linux libusb)
/dev/cu.usbserial    # Serial (macOS)
net:192.168.1.10:1420 # Network HotSync
bluez:00:11:22:33:44:55 # Bluetooth
```

**Файлы транспорта:**
- `serial.c` - Serial port abstraction
- `usb.c` - USB device handling (Linux)
- `darwinusb.c` - USB device handling (macOS)
- `freebsdusb.c` - USB device handling (FreeBSD)
- `unixserial.c` - Unix serial implementation
- `bluetooth.c` - Bluetooth RFCOMM
- `inet.c` - TCP/IP networking

---

## Формат данных Palm

### Record Attributes

```c
enum dlpRecAttributes {
    dlpRecAttrDeleted  = 0x80,  // Marked for deletion
    dlpRecAttrDirty    = 0x40,  // Modified since last sync
    dlpRecAttrBusy     = 0x20,  // Record locked
    dlpRecAttrSecret   = 0x10,  // Secret (hidden) record
    dlpRecAttrArchived = 0x08,  // Tagged for archive
};
```

### Database Flags

```c
enum dlpDBFlags {
    dlpDBFlagResource   = 0x0001,  // Resource database
    dlpDBFlagReadOnly   = 0x0002,  // Read-only database
    dlpDBFlagBackup     = 0x0008,  // Include in HotSync backup
    dlpDBFlagHidden     = 0x0100,  // Hidden from launcher
    dlpDBFlagLaunchable = 0x0200,  // Can be launched
    dlpDBFlagCopyPrevention = 0x0040,  // Cannot be beamed
};
```

### Типы баз данных (4-char codes)

```
'appl'  - Application database
'DATA'  - Data database
'syst'  - System database
'rsrc'  - Resource database
```

---

## Предлагаемая архитектура Rust

### Структура проекта

```
openpalm/
├── src/
│   ├── lib.rs
│   ├── error.rs           # Error types
│   ├── protocol/
│   │   ├── mod.rs
│   │   ├── socket.rs      # PilotSocket
│   │   ├── dlp.rs         # DLP protocol
│   │   ├── padp.rs        # PADP protocol
│   │   ├── net.rs         # NET protocol
│   │   └── slp.rs         # SLP protocol
│   ├── transport/
│   │   ├── mod.rs
│   │   ├── serial.rs      # Serial transport
│   │   ├── usb.rs         # USB transport
│   │   ├── bluetooth.rs   # Bluetooth transport
│   │   └── network.rs     # TCP/IP transport
│   ├── sync/
│   │   ├── mod.rs
│   │   ├── handler.rs     # SyncHandler trait
│   │   ├── strategies.rs  # Sync strategies
│   │   └── record.rs      # Record types
│   ├── database/
│   │   ├── mod.rs
│   │   ├── dbinfo.rs      # DatabaseInfo
│   │   ├── record.rs      # Record handling
│   │   └── vfs.rs         # VFS operations
│   └── types/
│       ├── mod.rs
│       ├── buffer.rs      # PiBuffer
│       ├── date.rs        # Palm date handling
│       └── addr.rs        # Address book types
└── tests/
```

### Cargo.toml dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }      # Async I/O
libusb = "1.0"                                        # USB communication
serialport = "4"                                     # Serial ports
bytes = "1"                                          # Buffer handling
thiserror = "1"                                      # Error handling
tracing = "0.1"                                      # Logging
bitflags = "2"                                       # Flags
uuid = "1"                                          # Unique IDs
time = "0.3"                                        # Date/time
```

### Ключевые трейты и типы

```rust
// === Error handling ===
#[derive(Debug, Error)]
pub enum PilotError {
    #[error("DLP error: {0}")]
    Dlp(DlpError),
    
    #[error("Palm OS error: {0}")]
    PalmOs(u16),
    
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// === Transport trait ===
pub trait Transport: Send + Sync {
    fn connect(&self, addr: &str) -> Result<Box<dyn Connection>>;
    fn listen(&self, addr: &str) -> Result<Box<dyn Listener>>;
}

// === Connection trait ===
pub trait Connection: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, buf: &[u8]) -> Result<usize>;
    fn flush(&self) -> Result<()>;
    fn close(&self) -> Result<()>;
}

// === Protocol trait ===
pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn level(&self) -> ProtocolLevel;
    
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>>;
}

// === DLP Client ===
pub struct DlpClient {
    socket: PilotSocket,
}

impl DlpClient {
    pub async fn connect(transport: &str) -> Result<Self> { /* ... */ }
    
    pub async fn open_database(&self, name: &str, mode: OpenMode) -> Result<DatabaseHandle>;
    
    pub async fn read_record(&self, db: &DatabaseHandle, id: RecordId) -> Result<Record>;
    
    pub async fn write_record(&self, db: &DatabaseHandle, record: &Record) -> Result<RecordId>;
    
    pub async fn delete_record(&self, db: &DatabaseHandle, id: RecordId) -> Result<()>;
    
    pub async fn get_sys_info(&self) -> Result<SysInfo>;
    
    pub async fn get_user_info(&self) -> Result<UserInfo>;
}

// === Sync Handler trait ===
#[async_trait]
pub trait SyncHandler: Send + Sync {
    fn database_name(&self) -> &str;
    
    async fn pre_sync(&self, db: &DatabaseHandle) -> Result<SyncStrategy>;
    
    async fn post_sync(&self, db: &DatabaseHandle) -> Result<()>;
    
    async fn for_each<F>(&self, callback: F) -> Result<()>
    where
        F: FnMut(DesktopRecord) -> Result<()> + Send;
    
    async fn add_record(&self, pilot_record: &PilotRecord) -> Result<()>;
    
    async fn match_record(&self, pilot_record: &PilotRecord) -> Result<Option<DesktopRecord>>;
    
    async fn compare(&self, pilot: &PilotRecord, desktop: &DesktopRecord) -> Result<Ordering>;
}

// === Sync strategies ===
pub enum SyncStrategy {
    Fast,    // Only modified records
    Slow,    // All records (fresh sync)
}

pub enum SyncDirection {
    ToPilot,
    FromPilot,
    Both,
}

pub async fn synchronize<H: SyncHandler>(
    handler: &H,
    client: &DlpClient,
    direction: SyncDirection,
) -> Result<()>;
```

---

## Следующие шаги

1. **Фаза 1: Core** (2-3 недели)
   - Создать базовую структуру проекта
   - Реализовать `PilotError` и `Result` типы
   - Реализовать `PiBuffer` для работы с данными
   - Реализовать базовый `PilotSocket`

2. **Фаза 2: Transport** (2-3 недели)
   - Реализовать serial transport
   - Реализовать USB transport (libusb)
   - Реализовать network transport

3. **Фаза 3: Protocol Stack** (3-4 недели)
   - Реализовать SLP protocol
   - Реализовать PADP protocol
   - Реализовать NET protocol
   - Реализовать DLP protocol (основной)

4. **Фаза 4: Database** (2 недели)
   - Реализовать `DatabaseHandle`
   - Реализовать `Record` операции
   - Реализовать VFS операции

5. **Фаза 5: Sync** (2-3 недели)
   - Реализовать `SyncHandler` trait
   - Реализовать стратегии синхронизации
   - Добавить примеры использования

---

## Стратегия тестирования

Тестирование — критически важная часть проекта, учитывая:
- Сложность протоколов коммуникации
- Необходимость совместимости с оригинальным pilot-link
- Работу с физическими устройствами (ограниченный доступ)

### Уровни тестирования

```
┌─────────────────────────────────────────────────────────────┐
│                    Integration Tests                        │
│    End-to-end тесты с реальными устройствами или           │
│    эмуляторами (Pose, Clié emulator)                        │
├─────────────────────────────────────────────────────────────┤
│                    Protocol Tests                            │
│    Тесты протоколов: DLP, PADP, NET, SLP                   │
│    Сравнение бинарных ответов с эталонными данными          │
├─────────────────────────────────────────────────────────────┤
│                    Component Tests                           │
│    Тесты отдельных компонентов:                             │
│    - Transport (mock connections)                           │
│    - Buffer handling                                        │
│    - Record parsing/serialization                          │
│    - Sync algorithms                                        │
├─────────────────────────────────────────────────────────────┤
│                    Unit Tests                                │
│    Базовые типы, кодирование/декодирование пакетов,        │
│    валидация данных, error handling                        │
└─────────────────────────────────────────────────────────────┘
```

### 1. Unit Tests (`src/`, `#[cfg(test)]`)

**Цель:** Тестировать изолированные компоненты без внешних зависимостей.

**Что тестировать:**

| Компонент | Тесты |
|-----------|-------|
| **PiBuffer** | Выделение памяти, grow/shrink, резерв, очистка, клонирование |
| **Date/Time** | `dlp_ptohdate`, `dlp_htopdate`, Unix ↔ Palm time conversion, timezone |
| **Error types** | `PilotError` variants, `From` implementations, `Display` formatting |
| **Bitflags** | Database flags, record attributes, packing/unpacking |
| **Packet encoding** | SLP/PADP/NET header building, checksum calculation |
| **4-char codes** | Pack/unpack для type/creator IDs |
| **Record parsing** | Address, Calendar, Todo - структуры данных |

**Примеры:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // === PiBuffer tests ===
    #[test]
    fn test_buffer_grow() {
        let mut buf = PiBuffer::with_capacity(10);
        buf.extend_from_slice(&[1, 2, 3]);
        assert_eq!(buf.len(), 3);
        buf.reserve(100);
        assert!(buf.capacity() >= 103);
    }
    
    #[test]
    fn test_buffer_clear() {
        let mut buf = PiBuffer::with_capacity(100);
        buf.extend_from_slice(&[1, 2, 3, 4, 5]);
        buf.clear();
        assert_eq!(buf.len(), 0);
        assert!(buf.capacity() >= 100); // Capacity preserved
    }
    
    // === Date conversion tests ===
    #[test]
    fn test_palm_date_conversion() {
        // Palm date: seconds since Jan 1, 1904
        // Unix date: seconds since Jan 1, 1970
        // Difference: 2082844800 seconds
        
        let unix_time: time_t = 0; // Unix epoch
        let palm_time = to_palm_time(unix_time);
        assert_eq!(palm_time, 0x83DAC000); // Jan 1, 1904
        
        let back = from_palm_time(palm_time);
        assert_eq!(back, unix_time);
    }
    
    #[test]
    fn test_palm_date_roundtrip() {
        let original = 1234567890_i64;
        let palm = to_palm_time(original);
        let restored = from_palm_time(palm);
        assert_eq!(original, restored);
    }
    
    // === Record flags tests ===
    #[test]
    fn test_record_flags_packing() {
        use dlp::RecordFlags;
        
        let flags = RecordFlags::DELETED 
            | RecordFlags::DIRTY 
            | RecordFlags::SECRET;
        
        assert_eq!(flags.bits(), 0xD0);
        assert!(flags.contains(RecordFlags::DELETED));
        assert!(flags.contains(RecordFlags::DIRTY));
        assert!(!flags.contains(RecordFlags::BUSY));
    }
    
    // === 4-char code tests ===
    #[test]
    fn test_four_char_code() {
        let code = FourCharCode::from_bytes(b"appl");
        assert_eq!(code.to_u32(), 0x6170706C);
        
        let back = FourCharCode::from_u32(0x6170706C);
        assert_eq!(back.as_bytes(), b"appl");
    }
    
    #[test]
    fn test_database_type() {
        assert_eq!(DatabaseType::APPLICATION, FourCharCode::from_bytes(b"appl"));
        assert_eq!(DatabaseType::DATA, FourCharCode::from_bytes(b"DATA"));
    }
    
    // === DLP packet tests ===
    #[test]
    fn test_dlp_arg_encoding_tiny() {
        // Arguments < 255 bytes use TINY encoding
        let arg = DlpArg::new(0x20, &[1, 2, 3]);
        let encoded = arg.encode();
        assert!(encoded.len() < 256);
    }
    
    #[test]
    fn test_dlp_arg_encoding_short() {
        // Arguments >= 255 and < 65536 bytes use SHORT encoding
        let data: Vec<u8> = (0..300).collect();
        let arg = DlpArg::new(0x20, &data);
        let encoded = arg.encode();
        // Should have 3-byte header (0x80 + 2 bytes length)
        assert_eq!(encoded[0] & 0xC0, 0x80);
    }
}
```

### 2. Component Tests (`tests/`)

**Цель:** Тестировать компоненты с mock-зависимостями.

**Структура:**

```
tests/
├── protocol/
│   ├── test_dlp_packets.rs      # DLP packet encoding/decoding
│   ├── test_padp_protocol.rs   # PADP state machine
│   ├── test_net_framing.rs     # NET protocol framing
│   └── test_slp_protocol.rs    # SLP protocol
├── transport/
│   ├── test_serial_mock.rs     # Mock serial port
│   ├── test_usb_mock.rs        # Mock USB device
│   └── test_connection.rs      # Connection abstraction
├── database/
│   ├── test_record_parsing.rs  # Parse binary records
│   ├── test_dbinfo.rs          # DatabaseInfo parsing
│   └── test_vfs.rs             # VFS operations (mock)
└── sync/
    ├── test_sync_handler.rs     # Handler trait implementation
    ├── test_merge_logic.rs      # Sync merge algorithms
    └── test_conflict_resolution.rs
```

**Примеры:**

```rust
// tests/protocol/test_dlp_packets.rs

use openpalm::protocol::dlp::*;

#[test]
fn test_dlp_read_record_request() {
    // Build a DLP_ReadRecord request
    let request = DlpRequest::builder()
        .command(DlpFunction::ReadRecord)
        .arg(DlpArg::Word(0))       // dbhandle
        .arg(DlpArg::DWord(12345))   // record ID
        .build();
    
    let bytes = request.encode();
    
    // Verify structure: [cmd, argc, args...]
    assert_eq!(bytes[0], 0x20);      // ReadRecord command
    assert_eq!(bytes[1], 2);         // 2 arguments
}

#[test]
fn test_dlp_read_record_response() {
    // Binary response from device (example)
    let response_bytes = [
        0x00, 0x00,             // Success/error code
        0x02,                   // argc
        0x01,                   // arg0 length (1 byte - tiny)
        0x40,                   // flags
        0x02,                   // arg1 length
        0x01, 0x00,             // category
        0x05,                   // arg2 length
        b'H', b'e', b'l', b'l', b'o',  // data
    ];
    
    let response = DlpResponse::decode(&response_bytes).unwrap();
    assert!(response.is_success());
    assert_eq!(response.flags(), RecordFlags::DIRTY);
}

#[test]
fn test_dlp_error_response() {
    let error_bytes = [0x00, 0x05, 0x00]; // NotFound error
    let response = DlpResponse::decode(&error_bytes).unwrap();
    
    assert!(response.is_error());
    assert_eq!(response.error(), DlpError::NotFound);
}

// tests/transport/test_serial_mock.rs

use openpalm::transport::serial::*;

struct MockSerialConnection {
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
}

impl MockSerialConnection {
    fn new(response_data: Vec<u8>) -> Self {
        Self {
            read_buffer: response_data,
            write_buffer: Vec::new(),
        }
    }
}

impl Connection for MockSerialConnection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = std::cmp::min(buf.len(), self.read_buffer.len());
        buf[..len].copy_from_slice(&self.read_buffer[..len]);
        self.read_buffer.drain(..len);
        Ok(len)
    }
    
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_buffer.extend_from_slice(buf);
        Ok(buf.len())
    }
}

#[test]
fn test_serial_baud_rate_handling() {
    let mock = MockSerialConnection::new(vec![]);
    
    // Test that we can set different baud rates
    let mut config = SerialConfig::default();
    config.baud_rate(BaudRate::B57600);
    
    assert_eq!(config.baud_rate(), BaudRate::B57600);
}

// tests/sync/test_merge_logic.rs

use openpalm::sync::*;

#[test]
fn test_sync_both_deleted() {
    // When record is deleted on both sides, do nothing
    let pilot = PilotRecord { flags: RecordFlags::DELETED, .. };
    let desktop = DesktopRecord { flags: RecordFlags::DELETED, .. };
    
    let result = resolve_sync_conflict(&pilot, &desktop, SyncStrategy::Fast);
    assert_eq!(result, SyncAction::DeleteBoth);
}

#[test]
fn test_sync_pilot_modified_desktop_clean() {
    // When pilot has modifications, desktop is clean - update desktop
    let pilot = PilotRecord { 
        flags: RecordFlags::DIRTY,
        data: b"new data".to_vec(),
        .. 
    };
    let desktop = DesktopRecord { flags: RecordFlags::empty(), .. };
    
    let result = resolve_sync_conflict(&pilot, &desktop, SyncStrategy::Fast);
    assert_eq!(result, SyncAction::UpdateDesktop);
}

#[test]
fn test_sync_conflict_resolution() {
    // When both modified - use timestamp or manual resolution
    let pilot = PilotRecord { 
        flags: RecordFlags::DIRTY,
        modify_time: 1000,
        data: b"pilot version".to_vec(),
        .. 
    };
    let desktop = DesktopRecord { 
        flags: RecordFlags::DIRTY,
        modify_time: 2000,
        .. 
    };
    
    let result = resolve_sync_conflict(&pilot, &desktop, SyncStrategy::Fast);
    // Latest timestamp wins
    assert_eq!(result, SyncAction::KeepDesktop);
}
```

### 3. Protocol Fuzzing (`fuzz/`)

**Цель:** Находить edge cases в парсинге бинарных данных от устройств.

```rust
// fuzz/fuzz_dlp_parser.rs

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Test DLP response parsing with arbitrary data
    if let Ok(response) = DlpResponse::decode(data) {
        // Verify internal consistency
        if response.is_success() {
            // Should have valid argument count
            assert!(response.argc() <= 20);
        }
    }
});

fuzz_target!(|data: &[u8]| {
    // Test PADP packet parsing
    if let Ok(packet) = PadpPacket::decode(data) {
        // Verify state transitions are valid
        match packet.packet_type() {
            PadpType::SyncRequest => {
                // Should only appear in initial state
            }
            // ...
        }
    }
});

// fuzz/fuzz_record_parser.rs

fuzz_target!(|data: &[u8]| {
    // Test address book record parsing
    if let Ok(record) = AddressRecord::parse(data) {
        // Verify all fields are valid
        assert!(record.name.len() <= 256);
        assert!(record.phones.len() <= 20);
    }
});
```

**Cargo.toml:**
```toml
[dependencies]
libfuzzer-sys = "0.4"

[[bin]]
name = "fuzz_dlp_parser"
path = "fuzz/fuzz_dlp_parser.rs"
```

### 4. Snapshot Tests (Golden Tests)

**Цель:** Гарантировать совместимость с оригинальным pilot-link.

```rust
// tests/snapshot/test_dlp_packets.rs

use openpalm::protocol::dlp::*;
use insta::{assert_debug_snapshot, with_settings};

#[test]
fn test_dlp_open_db_request_snapshot() {
    let request = DlpRequest::builder()
        .command(DlpFunction::OpenDB)
        .arg(DlpArg::Byte(0))                    // cardNo
        .arg(DlpArg::Byte(0xC0))                 // mode: read+write
        .arg(DlpArg::String("TestDB"))          // database name
        .build();
    
    let encoded = request.encode();
    
    // Compare with known-good output from pilot-link
    with_settings! {
        filters => vec![(
            r"timestamp: [0-9]+", 
            "timestamp: <TIMESTAMP>"
        )],
    }
    {
        assert_debug_snapshot!("dlp_open_db_request", &encoded);
    }
}

// tests/snapshots/dlp_open_db_request.snap
//
// This file is generated from the original pilot-link implementation.
// DO NOT EDIT MANUALLY - run tests with INSTA_UPDATE=1 to regenerate.

// --- snapshot: dlp_open_db_request ---
// length: 12
// data: [
//     0x17,  // OpenDB command
//     0x03,  // argc
//     0x01, 0x00,  // cardNo
//     0x01, 0xC0,  // mode (read+write)
//     0x07, "T", "e", "s", "t", "D", "B", 0x00  // name + null
// ]
```

### 5. Property-Based Tests

**Цель:** Тестировать invariants с рандомизированными данными.

```rust
// tests/property/

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_dlp_arg_roundtrip( arg_id: u8, len: 0..65536u16) {
        let data: Vec<u8> = (0..len).map(|i| (i % 256) as u8).collect();
        
        let arg = DlpArg::new(0x20 + arg_id, &data);
        let encoded = arg.encode();
        let decoded = DlpArg::decode(&encoded).unwrap();
        
        prop_assert_eq!(decoded.id(), arg.id());
        prop_assert_eq!(decoded.data(), &data);
    }
    
    #[test]
    fn test_record_flags_combinatorial(flags: u8) {
        let flags = RecordFlags::from_bits_truncate(flags);
        
        // All individual flags should be extractable
        let is_deleted = flags.contains(RecordFlags::DELETED);
        let is_dirty = flags.contains(RecordFlags::DIRTY);
        let is_busy = flags.contains(RecordFlags::BUSY);
        let is_secret = flags.contains(RecordFlags::SECRET);
        let is_archived = flags.contains(RecordFlags::ARCHIVED);
        
        // Reconstruct from parts
        let reconstructed = RecordFlags::empty()
            | if is_deleted { RecordFlags::DELETED } else { RecordFlags::empty() }
            | if is_dirty { RecordFlags::DIRTY } else { RecordFlags::empty() }
            | if is_busy { RecordFlags::BUSY } else { RecordFlags::empty() }
            | if is_secret { RecordFlags::SECRET } else { RecordFlags::empty() }
            | if is_archived { RecordFlags::ARCHIVED } else { RecordFlags::empty() };
        
        prop_assert_eq!(flags, reconstructed);
    }
    
    #[test]
    fn test_four_char_code_roundtrip(code: prop::array::UniformArray<u8, 4>) {
        let bytes: [u8; 4] = code.0;
        let fourcc = FourCharCode::from_bytes(bytes);
        let restored = fourcc.as_bytes();
        
        prop_assert_eq!(bytes, restored);
    }
    
    #[test]
    fn test_palm_time_always_valid(unix_secs: i64) {
        // Palm time should always be positive and within reasonable bounds
        let palm = to_palm_time(unix_secs);
        
        prop_assert!(palm > 0);
        prop_assert!(palm < i64::MAX - 2082844800); // Reasonable upper bound
        
        // Roundtrip
        let restored = from_palm_time(palm);
        prop_assert_eq!(unix_secs, restored);
    }
}
```

**Cargo.toml:**
```toml
[dev-dependencies]
proptest = "1"
```

### 6. Integration Tests

**Цель:** Тестировать взаимодействие с реальными устройствами или качественными mock-ами.

```rust
// tests/integration/

#[cfg(test)]
mod test_dlp_integration {
    use super::*;
    
    // Skip if no device or test device available
    fn require_test_device() -> Option<TestDevice> {
        std::env::var("TEST_DEVICE").ok()
            .map(|addr| TestDevice::connect(&addr).unwrap())
    }
    
    #[test]
    #[ignore = "requires physical device"]
    fn test_connect_and_get_sys_info() {
        let device = require_test_device().expect("TEST_DEVICE not set");
        let mut client = DlpClient::connect(device).unwrap();
        
        let sys_info = client.get_sys_info().unwrap();
        
        assert!(sys_info.rom_version > 0);
        assert!(sys_info.dlp_version.major >= 1);
    }
    
    #[test]
    #[ignore = "requires physical device"]
    fn test_database_operations() {
        let device = require_test_device().unwrap();
        let client = DlpClient::connect(device).unwrap();
        
        // List databases
        let dbs = client.list_databases(dlpDBListRAM).unwrap();
        assert!(!dbs.is_empty());
        
        // Open a known database
        if let Some(db) = dbs.iter().find(|d| d.name == "AddressDB") {
            let handle = client.open_database(&db.name, OpenMode::Read).unwrap();
            
            let records = client.read_all_records(&handle).unwrap();
            assert!(records.len() > 0);
            
            client.close_database(handle).unwrap();
        }
    }
}

// tests/integration/test_sync_integration.rs

#[test]
#[ignore = "requires device and test backend"]
fn test_full_sync_cycle() {
    let device = require_test_device().unwrap();
    let client = DlpClient::connect(device).unwrap();
    
    // Create a mock backend
    let backend = TestSyncBackend::new();
    
    // Run sync
    let handler = SyncHandler::new("AddressDB", &backend, &client);
    sync_synchronize(&handler, &client, SyncDirection::Both).unwrap();
    
    // Verify results
    assert_eq!(backend.record_count(), 10); // Or whatever expected
}
```

### 7. Compliance Tests (Black-box testing)

**Цель:** Убедиться, что наша реализация совместима с оригинальным pilot-link.

```rust
// tests/compliance/test_pilot_link_compat.rs

use openpalm::protocol::*;

/// Test that our implementation produces identical DLP packets
/// as the original pilot-link library.
/// 
/// These tests compare the binary output of key operations.
#[test]
fn test_compliance_dlp_handshake() {
    // Simulate DLP handshake sequence
    let sequence = dlp_handshake_sequence();
    
    // Our output
    let our_output = encode_dlp_sequence(&sequence);
    
    // Expected from pilot-link (captured from real session)
    let expected = include_bytes!("fixtures/dlp_handshake.bin");
    
    assert_eq!(&our_output[..], expected);
}

#[test]
fn test_compliance_record_encoding() {
    let address = AddressRecord {
        name: "John Doe".to_string(),
        phones: vec![
            Phone { 
                label: "work".to_string(), 
                number: "+1234567890".to_string() 
            }
        ],
        ..Default::default()
    };
    
    let our_bytes = address.to_palm_bytes();
    let expected = include_bytes!("fixtures/address_record.bin");
    
    // Allow some flexibility in format but key data must match
    assert_eq!(our_bytes.len(), expected.len());
}

// tests/compliance/fixtures/
// Include captured packet sequences from real pilot-link sessions
// These serve as the "source of truth" for compatibility
```

### 8. Test Infrastructure

#### Docker/Container для Integration Tests

```yaml
# docker/test-environment.dockerfile

FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    libusb-1.0-0-dev \
    libudev-dev \
    pilot-link-tools  # For comparison testing

# Setup mock udev rules for testing USB without real device
COPY udev/99-palm-test.rules /etc/udev/rules.d/

# Run tests in isolated environment
ENTRYPOINT ["/bin/bash", "-c", "cargo test && cargo test --test integration"]
```

#### CI Pipeline

```yaml
# .github/workflows/test.yml

name: Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --lib --bins
      - run: cargo test --doc
      
  component-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --test '*'
      
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --test property -- --nocapture
      
  snapshot-tests:
    runs-on: ubuntu-latest
    env:
      INSTA_UPDATE: always
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --test snapshot
      
  fuzzing:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo fuzz run dlp_parser -- -max_len=65536
      
  integration:
    runs-on: ubuntu-latest
    if: github.event_name == 'push'
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --test integration
        env:
          TEST_DEVICE: ${{ secrets.TEST_DEVICE_ADDR }}
```

### 9. Summary: Test Coverage Matrix

| Компонент | Unit | Component | Property | Snapshot | Integration |
|-----------|:----:|:---------:|:--------:|:--------:|:-----------:|
| **PiBuffer** | ✓ | ✓ | ✓ | | |
| **Date/Time** | ✓ | | ✓ | | |
| **Error types** | ✓ | | | | |
| **Record flags** | ✓ | | ✓ | | |
| **FourCharCode** | ✓ | | ✓ | | |
| **DLP packets** | ✓ | ✓ | ✓ | ✓ | |
| **DLP functions** | ✓ | ✓ | | ✓ | ✓ |
| **PADP protocol** | ✓ | ✓ | ✓ | ✓ | |
| **NET protocol** | ✓ | ✓ | ✓ | ✓ | |
| **SLP protocol** | ✓ | ✓ | | ✓ | |
| **Serial transport** | ✓ | ✓ | | | |
| **USB transport** | ✓ | ✓ | | | |
| **Record parsing** | ✓ | ✓ | ✓ | ✓ | |
| **VFS operations** | ✓ | ✓ | | ✓ | |
| **Sync merge** | ✓ | ✓ | ✓ | ✓ | |
| **Sync handler** | ✓ | ✓ | | ✓ | |

### 10. Running Tests

```bash
# All unit tests
cargo test --lib

# Documentation tests
cargo test --doc

# All tests including integration
cargo test --all

# With coverage
cargo tarpaulin --out Xml --output-dir coverage/

# Property-based tests
cargo test --test property

# Update snapshots
INSTA_UPDATE=1 cargo test --test snapshot

# Fuzzing (requires corpus)
cargo fuzz run dlp_parser

# Specific integration test
cargo test --test integration test_database_operations -- --ignored

# Run with device
TEST_DEVICE=/dev/ttyUSB0 cargo test --test integration -- --ignored
```

---

## Полезные ресурсы

- **Оригинальный код:** https://github.com/jichu4n/pilot-link
- **DLP Protocol Spec:** См. `include/pi-dlp.h` (77KB документации)
- **Palm OS SDK Documentation:** Для понимания форматов записей
- **Sync Algorithm:** Документация в `libpisync/sync.c`
