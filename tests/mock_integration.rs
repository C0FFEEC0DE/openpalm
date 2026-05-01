//! Integration tests using MockConnection for end-to-end DLP flow.
//!
//! These tests exercise the full stack: PilotSocket → DlpClient → MockConnection
//! with realistic request/response parsing.

use openpalm::{
    PilotSocket,
    protocol::dlp::{DlpFunction, DlpErrorCode},
    protocol::TransportConnection,
};

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
    data.extend_from_slice(&0x01020304u32.to_le_bytes());

    // Arg 1: locale = 0x00000001 (u32, LE)
    data.push(0x04);
    data.extend_from_slice(&0x00000001u32.to_le_bytes());

    // Arg 2: prod_id_len = 4 (u8)
    data.push(0x01);
    data.push(0x04);

    // Arg 3: prod_id = "Test\0" (5 bytes)
    data.push(0x05);
    data.extend_from_slice(b"Test\0");

    // Arg 4: dlp_major = 1 (u16, LE)
    data.push(0x02);
    data.extend_from_slice(&1u16.to_le_bytes());

    // Arg 5: dlp_minor = 4 (u16, LE)
    data.push(0x02);
    data.extend_from_slice(&4u16.to_le_bytes());

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
    data.extend_from_slice(&0x30295296u32.to_le_bytes());

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
