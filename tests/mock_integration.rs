//! Integration tests using MockConnection for end-to-end DLP flow.
//!
//! These tests exercise the full stack: PilotSocket → DlpClient → MockConnection
//! with realistic request/response parsing.

use openpalm::{
    PilotSocket,
    protocol::dlp::{DlpFunction, DlpErrorCode, DlpOpenMode, DlpDBListFlag},
    protocol::TransportConnection,
    types::{FourCharCode, RecordFlags, DatabaseFlags},
};

// ========================================================================
// System Info Tests
// ========================================================================

/// Build a raw DLP response packet for `ReadSysInfo`.
///
/// Wire format: [function, argc, error_code, flags] + body (args)
fn build_read_sys_info_response() -> Vec<u8> {
    let mut data = vec![
        DlpFunction::ReadSysInfo as u8, // 0x12
        6,                              // argc
        DlpErrorCode::NoError as u8,    // 0x00
        0,                              // flags
    ];

    // Arg 0: rom_version = 0x01020304 (u32, LE)
    data.push(0x04); // tiny format, len=4
    data.extend_from_slice(&0x01020304u32.to_be_bytes());

    // Arg 1: locale = 0x00000001 (u32, LE)
    data.push(0x04);
    data.extend_from_slice(&0x00000001u32.to_be_bytes());

    // Arg 2: prod_id_len = 4 (u8)
    data.push(0x01);
    data.push(0x04);

    // Arg 3: prod_id = "Test\0" (5 bytes)
    data.push(0x05);
    data.extend_from_slice(b"Test\0");

    // Arg 4: dlp_major = 1 (u16, LE)
    data.push(0x02);
    data.extend_from_slice(&1u16.to_be_bytes());

    // Arg 5: dlp_minor = 4 (u16, LE)
    data.push(0x02);
    data.extend_from_slice(&4u16.to_be_bytes());

    data
}

#[tokio::test]
async fn test_mock_read_sys_info() {
    let mut socket = PilotSocket::mock();

    // Pre-populate mock transport with a valid DLP response
    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_read_sys_info_response());
    }

    socket.connect().unwrap();
    assert!(socket.is_connected());

    let dlp = socket.dlp().unwrap();
    let sys_info = dlp.read_sys_info().await.unwrap();

    assert_eq!(sys_info.rom_version, 0x01020304);
    assert_eq!(sys_info.locale, 0x00000001);
    assert_eq!(sys_info.prod_id_len, 4);
    assert_eq!(sys_info.prod_id, "Test");
    assert_eq!(sys_info.dlp_major, 1);
    assert_eq!(sys_info.dlp_minor, 4);
    assert_eq!(sys_info.max_rec_size, 0xFFFF);
}

/// Build a raw DLP response with an error code.
fn build_error_response(function: DlpFunction, error: DlpErrorCode) -> Vec<u8> {
    vec![function as u8, 0, error as u8, 0] // + flags byte
}

#[tokio::test]
async fn test_error_response_trailing_data_consumed() {
    let mut socket = PilotSocket::mock();

    // Two error responses back-to-back; first has trailing argv bytes that must be consumed
    let mut data = build_error_response(
        DlpFunction::ReadSysInfo,
        DlpErrorCode::NotFound,
    );
    data.extend_from_slice(&[0x01, 0x02, 0x03]); // trailing argv bytes
    data.extend_from_slice(&build_error_response(
        DlpFunction::ReadSysInfo,
        DlpErrorCode::NotFound,
    ));

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(data);
        mock.set_chunk_size(1); // force byte-by-byte reads so body does not over-read
        mock.set_read_limit(7); // 4 header + 3 trailing — body read stops here
    }

    socket.connect().unwrap();

    // First call: ReadSysInfo error
    let err1 = socket.read_sys_info().await.unwrap_err();
    assert!(matches!(err1, openpalm::PilotError::DlpError(5)));

    // Allow second response to be read (must clear limit on the DlpClient's transport clone)
    {
        let client = socket.dlp().unwrap();
        client.with_transport_mut(|conn| {
            if let TransportConnection::Mock(mock) = conn {
                mock.clear_read_limit();
            }
        });
    }

    // Second call: must also return NotFound (not DlpError(3) from stale trailing bytes)
    let err2 = socket.read_sys_info().await.unwrap_err();
    assert!(matches!(err2, openpalm::PilotError::DlpError(5)));
}

#[tokio::test]
async fn test_mock_read_sys_info_error() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_error_response(
            DlpFunction::ReadSysInfo,
            DlpErrorCode::NotFound,
        ));
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let err = dlp.read_sys_info().await.unwrap_err();

    // The error should be a DLP error (NotFound = 0x05)
    match err {
        openpalm::PilotError::DlpError(code) => assert_eq!(code, DlpErrorCode::NotFound as u16),
        other => panic!("expected DlpError, got {:?}", other),
    }
}

/// Build a raw DLP response for `GetSysDateTime`.
fn build_get_sys_datetime_response() -> Vec<u8> {
    let mut data = vec![
        DlpFunction::GetSysDateTime as u8, // 0x20
        1,                                  // argc
        DlpErrorCode::NoError as u8,        // 0x00
        0,                                  // flags
    ];

    // Arg 0: palm datetime = 0x30295296 (u32, LE)
    data.push(0x04);
    data.extend_from_slice(&0x30295296u32.to_be_bytes());

    data
}

#[tokio::test]
async fn test_mock_get_sys_datetime() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_get_sys_datetime_response());
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let dt = dlp.get_sys_datetime().await.unwrap();

    // 0x30295296 seconds since Palm epoch (1904-01-01)
    assert_eq!(dt.to_palm(), 0x30295296);
}

/// Build a raw DLP response for `ReadStorageInfo`.
fn build_read_storage_info_response() -> Vec<u8> {
    let mut data = vec![
        DlpFunction::ReadStorageInfo as u8, // 0x15
        6,                                     // argc
        DlpErrorCode::NoError as u8,          // 0x00
        0,                                    // flags
    ];

    // Arg 0: version = 1 (i32, LE)
    data.push(0x04);
    data.extend_from_slice(&1i32.to_be_bytes());

    // Arg 1: rom_size = 0x00100000 (u32, LE) = 1MB
    data.push(0x04);
    data.extend_from_slice(&0x00100000u32.to_be_bytes());

    // Arg 2: ram_size = 0x00080000 (u32, LE) = 512KB
    data.push(0x04);
    data.extend_from_slice(&0x00080000u32.to_be_bytes());

    // Arg 3: ram_free = 0x00040000 (u32, LE) = 256KB
    data.push(0x04);
    data.extend_from_slice(&0x00040000u32.to_be_bytes());

    // Arg 4: name = "Palm\0" (5 bytes)
    data.push(0x05);
    data.extend_from_slice(b"Palm\0");

    // Arg 5: manufacturer = "Palm Inc\0" (9 bytes)
    data.push(0x09);
    data.extend_from_slice(b"Palm Inc\0");

    data
}

#[tokio::test]
async fn test_mock_read_storage_info() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_read_storage_info_response());
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let info = dlp.read_storage_info(0).await.unwrap();

    assert_eq!(info.version, 1);
    assert_eq!(info.rom_size, 0x00100000);
    assert_eq!(info.ram_size, 0x00080000);
    assert_eq!(info.ram_free, 0x00040000);
    assert_eq!(info.name, "Palm");
    assert_eq!(info.manufacturer, "Palm Inc");
}

/// Build a raw DLP response for `ReadUserInfo`.
fn build_read_user_info_response() -> Vec<u8> {
    let mut data = vec![
        DlpFunction::ReadUserInfo as u8, // 0x10
        4,                                  // argc
        DlpErrorCode::NoError as u8,     // 0x00
        0,                                 // flags
    ];

    // Arg 0: user_id = 0x12345678 (u32, LE)
    data.push(0x04);
    data.extend_from_slice(&0x12345678u32.to_be_bytes());

    // Arg 1: viewer_id = 0x00000000 (u32, LE)
    data.push(0x04);
    data.extend_from_slice(&0x00000000u32.to_be_bytes());

    // Arg 2: username = "TestUser\0" (9 bytes)
    data.push(0x09);
    data.extend_from_slice(b"TestUser\0");

    // Arg 3: last_sync_pc = 0x00 (u32, LE)
    data.push(0x04);
    data.extend_from_slice(&0u32.to_be_bytes());

    data
}

#[tokio::test]
async fn test_mock_read_user_info() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_read_user_info_response());
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let user = dlp.read_user_info().await.unwrap();

    assert_eq!(user.user_id, 0x12345678);
    assert_eq!(user.viewer_id, 0);
    assert_eq!(user.username, "TestUser");
    assert_eq!(user.last_sync_pc, 0);
}

// ========================================================================
// Database Function Tests
// ========================================================================

/// Build a raw DLP response for `OpenDB`.
fn build_open_db_response(handle: u8) -> Vec<u8> {
    let mut data = vec![
        DlpFunction::OpenDB as u8,  // 0x17
        1,                          // argc
        DlpErrorCode::NoError as u8, // 0x00
        0,                          // flags
    ];
    // Arg 0: handle (u32, LE)
    data.push(0x04);
    data.extend_from_slice(&(handle as u32).to_be_bytes());
    data
}

#[tokio::test]
async fn test_mock_open_db() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_open_db_response(5));
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let handle = dlp.open_db(0, "TestDB", DlpOpenMode::ReadWrite).await.unwrap();

    assert_eq!(handle, 5);
}

#[tokio::test]
async fn test_mock_open_db_error() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_error_response(
            DlpFunction::OpenDB,
            DlpErrorCode::NotFound,
        ));
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let err = dlp.open_db(0, "NonExistent", DlpOpenMode::ReadWrite).await.unwrap_err();

    match err {
        openpalm::PilotError::DlpError(code) => assert_eq!(code, DlpErrorCode::NotFound as u16),
        other => panic!("expected DlpError, got {:?}", other),
    }
}

/// Build a raw DLP response for `CloseDB`.
fn build_close_db_response() -> Vec<u8> {
    vec![
        DlpFunction::CloseDB as u8,
        0,                              // argc
        DlpErrorCode::NoError as u8,
        0,                              // flags
    ]
}

#[tokio::test]
async fn test_mock_close_db() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_close_db_response());
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    dlp.close_db(5).await.unwrap();
    // If we got here without error, the test passed
}

/// Build a raw DLP response for `ReadDBList`.
fn build_read_db_list_response() -> Vec<u8> {
    let mut data = vec![
        DlpFunction::ReadDBList as u8, // 0x16
        28,                             // argc (2 databases * 14 args each)
        DlpErrorCode::NoError as u8,
        0,
    ];

    // Helper to add a tiny-format argument (length byte + data)
    let add_tiny = |data: &mut Vec<u8>, val: &[u8]| {
        data.push(val.len() as u8); // length byte (must have MSB clear for tiny format)
        data.extend_from_slice(val);
    };

    // Database 1: "AddrDB" (8 bytes with null)
    add_tiny(&mut data, b"AddrDB\0");                    // arg 0: name
    add_tiny(&mut data, &0x0001u16.to_be_bytes());       // arg 1: flags
    add_tiny(&mut data, &0x44415442u32.to_be_bytes());  // arg 2: db_type "DATB"
    add_tiny(&mut data, &0x50414C4Du32.to_be_bytes());  // arg 3: creator "PALM"
    add_tiny(&mut data, &[0]);                           // arg 4: card_no
    add_tiny(&mut data, &1u32.to_be_bytes());            // arg 5: db_id
    add_tiny(&mut data, &0x30000000u32.to_be_bytes());  // arg 6: created
    add_tiny(&mut data, &0x30100000u32.to_be_bytes());  // arg 7: modified
    add_tiny(&mut data, &0u32.to_be_bytes());            // arg 8: backup_date
    add_tiny(&mut data, &100u32.to_be_bytes());         // arg 9: mod_num
    add_tiny(&mut data, &0x00004000u32.to_be_bytes());  // arg 10: total_bytes
    add_tiny(&mut data, &0x00003000u32.to_be_bytes());  // arg 11: data_bytes
    add_tiny(&mut data, &25u16.to_be_bytes());           // arg 12: num_records
    add_tiny(&mut data, &1u32.to_be_bytes());            // arg 13: unique_id_seed

    // Database 2: "DateBkDB" (8 bytes with null)
    add_tiny(&mut data, b"DateBkDB\0");                  // arg 14: name
    add_tiny(&mut data, &0x0001u16.to_be_bytes());       // arg 15: flags
    add_tiny(&mut data, &0x44415442u32.to_be_bytes());  // arg 16: db_type
    add_tiny(&mut data, &0x50414C4Du32.to_be_bytes());  // arg 17: creator
    add_tiny(&mut data, &[0]);                           // arg 18: card_no
    add_tiny(&mut data, &2u32.to_be_bytes());            // arg 19: db_id
    add_tiny(&mut data, &0x30010000u32.to_be_bytes());  // arg 20: created
    add_tiny(&mut data, &0x30120000u32.to_be_bytes());  // arg 21: modified
    add_tiny(&mut data, &0u32.to_be_bytes());            // arg 22: backup_date
    add_tiny(&mut data, &200u32.to_be_bytes());         // arg 23: mod_num
    add_tiny(&mut data, &0x00008000u32.to_be_bytes());  // arg 24: total_bytes
    add_tiny(&mut data, &0x00006000u32.to_be_bytes());  // arg 25: data_bytes
    add_tiny(&mut data, &50u16.to_be_bytes());           // arg 26: num_records
    add_tiny(&mut data, &1u32.to_be_bytes());            // arg 27: unique_id_seed

    data
}

#[tokio::test]
async fn test_mock_read_db_list() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_read_db_list_response());
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let databases = dlp.read_db_list(0, DlpDBListFlag::Ram, 0).await.unwrap();

    assert_eq!(databases.len(), 2);

    assert_eq!(databases[0].name, "AddrDB");
    assert_eq!(databases[0].creator.to_u32(), 0x50414C4D);
    assert_eq!(databases[0].card_no, 0);
    assert_eq!(databases[0].num_records, 25);

    assert_eq!(databases[1].name, "DateBkDB");
    assert_eq!(databases[1].creator.to_u32(), 0x50414C4D);
    assert_eq!(databases[1].card_no, 0);
    assert_eq!(databases[1].num_records, 50);
}

/// Build a raw DLP response for `CreateDB`.
fn build_create_db_response(handle: u8) -> Vec<u8> {
    let mut data = vec![
        DlpFunction::CreateDB as u8, // 0x18
        1,                           // argc
        DlpErrorCode::NoError as u8,
        0,
    ];
    data.push(0x04);
    data.extend_from_slice(&(handle as u32).to_be_bytes());
    data
}

#[tokio::test]
async fn test_mock_create_db() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_create_db_response(7));
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let creator = FourCharCode::from_u32(0x50414C4D);
    let db_type = FourCharCode::from_u32(0x44415442);
    let handle = dlp.create_db(creator, db_type, 0, DatabaseFlags::empty(), 1, "NewDB")
        .await
        .unwrap();

    assert_eq!(handle, 7);
}

/// Build a raw DLP response for `DeleteDB`.
fn build_delete_db_response() -> Vec<u8> {
    vec![
        DlpFunction::DeleteDB as u8,
        0,
        DlpErrorCode::NoError as u8,
        0,
    ]
}

#[tokio::test]
async fn test_mock_delete_db() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_delete_db_response());
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    dlp.delete_db(0, "OldDB").await.unwrap();
}

#[tokio::test]
async fn test_mock_delete_db_error() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_error_response(
            DlpFunction::DeleteDB,
            DlpErrorCode::ReadOnly,
        ));
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let err = dlp.delete_db(0, "ReadOnlyDB").await.unwrap_err();

    match err {
        openpalm::PilotError::DlpError(code) => assert_eq!(code, DlpErrorCode::ReadOnly as u16),
        other => panic!("expected DlpError, got {:?}", other),
    }
}

// ========================================================================
// Record Function Tests
// ========================================================================

/// Build a raw DLP response for `ReadRecord`.
fn build_read_record_response(id: u32, index: u32, data: &[u8], attrs: u8) -> Vec<u8> {
    let mut response = vec![
        DlpFunction::ReadRecord as u8, // 0x20
        4,                             // argc
        DlpErrorCode::NoError as u8,
        0,
    ];

    // Arg 0: record data
    response.push(data.len() as u8);
    response.extend_from_slice(data);

    // Arg 1: record id (u32)
    response.push(0x04);
    response.extend_from_slice(&id.to_be_bytes());

    // Arg 2: attributes (u8)
    response.push(0x01);
    response.push(attrs);

    // Arg 3: index (u32)
    response.push(0x04);
    response.extend_from_slice(&index.to_be_bytes());

    response
}

#[tokio::test]
async fn test_mock_read_record() {
    let mut socket = PilotSocket::mock();

    let record_data = b"Hello, Palm!";
    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_read_record_response(0x10000001, 5, record_data, 0x00));
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let record = dlp.read_record(1, 5).await.unwrap();

    assert_eq!(record.id, 0x10000001);
    assert_eq!(record.index, 5);
    assert_eq!(record.data, record_data);
}

/// Build a raw DLP response for `WriteRecord`.
fn build_write_record_response(id: u32) -> Vec<u8> {
    let mut data = vec![
        DlpFunction::WriteRecord as u8, // 0x21
        1,                               // argc
        DlpErrorCode::NoError as u8,
        0,
    ];
    data.push(0x04);
    data.extend_from_slice(&id.to_be_bytes());
    data
}

#[tokio::test]
async fn test_mock_write_record() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_write_record_response(0x10000005));
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let id = dlp.write_record(1, RecordFlags::empty(), 0, 0, b"New record data")
        .await
        .unwrap();

    assert_eq!(id, 0x10000005);
}

/// Build a raw DLP response for `DeleteRecord`.
fn build_delete_record_response() -> Vec<u8> {
    vec![
        DlpFunction::DeleteRecord as u8,
        0,
        DlpErrorCode::NoError as u8,
        0,
    ]
}

#[tokio::test]
async fn test_mock_delete_record() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_delete_record_response());
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    dlp.delete_record(1, 5, 0x10000001).await.unwrap();
}

// ========================================================================
// Request Verification Test
// ========================================================================

/// Verify that the request is written correctly to the mock transport.
#[tokio::test]
async fn test_mock_request_written() {
    let mut socket = PilotSocket::mock();

    if let Some(TransportConnection::Mock(mock)) = socket.transport_mut() {
        mock.set_read_data(build_read_sys_info_response());
    }

    socket.connect().unwrap();

    let dlp = socket.dlp().unwrap();
    let _ = dlp.read_sys_info().await.unwrap();

    // Re-acquire transport to inspect written data
    let transport = dlp.transport();
    let mut guard = transport.lock().unwrap();
    let written = match &mut *guard {
        TransportConnection::Mock(mock) => mock.written_data().to_vec(),
        _ => panic!("expected Mock connection"),
    };

    // The request should be a DLP ReadSysInfo request: [function, argc=0]
    assert_eq!(written.len(), 2);
    assert_eq!(written[0], DlpFunction::ReadSysInfo as u8);
    assert_eq!(written[1], 0);
}