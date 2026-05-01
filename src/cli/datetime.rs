//! Device datetime commands

use crate::error::{PilotError, Result};
use crate::PilotSocket;
use crate::types::PalmDateTime;

/// Show device datetime
pub async fn show(socket: &mut PilotSocket) -> Result<()> {
    let dt = socket.dlp().ok_or(PilotError::DlpSocket)?.get_sys_datetime().await?;
    let unix = dt.to_unix();
    println!("Device time: {:?} (Unix: {})", dt, unix);
    Ok(())
}

/// Set device datetime to current system time
pub async fn set_now(socket: &mut PilotSocket) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let palm_dt = PalmDateTime::from_unix(now);
    socket.dlp().ok_or(PilotError::DlpSocket)?.set_sys_datetime(palm_dt).await?;
    println!("Device time set to current system time.");
    Ok(())
}
