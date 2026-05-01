//! Resource commands

use crate::error::{PilotError, Result};
use crate::PilotSocket;
use crate::cli::print_table;
use crate::protocol::dlp::DlpOpenMode;

/// List resources in a database
pub async fn list(socket: &mut PilotSocket, db_name: &str) -> Result<()> {
    let handle = socket.open_database(db_name, DlpOpenMode::Read).await?;
    let db_info = socket.dlp().ok_or(PilotError::DlpSocket)?.read_open_db_info(0, handle).await?;
    
    // Resource databases have record-like structure
    println!("Database: {} ({} resources)", db_name, db_info.1.num_records);
    
    socket.close_database(handle).await?;
    Ok(())
}
