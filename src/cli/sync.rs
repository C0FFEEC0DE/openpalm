//! Sync commands

use crate::error::{PilotError, Result};
use crate::PilotSocket;
use crate::cli::print_table;

/// Perform a sync with the device
pub async fn sync_device(socket: &mut PilotSocket) -> Result<()> {
    println!("Opening conduit...");
    socket.dlp().ok_or(PilotError::DlpSocket)?.open_conduit().await?;
    
    // Get list of databases and sync
    let dbs = socket.list_databases().await?;
    println!("Found {} databases", dbs.len());
    
    let mut rows = Vec::new();
    for db in &dbs {
        // Open database and read modified records
        let handle = socket.open_database(&db.name, crate::protocol::dlp::DlpOpenMode::Read).await?;
        let mut modified_count = 0u32;
        
        // Try to read next modified record
        while let Ok(Some(_record)) = socket.dlp().ok_or(PilotError::DlpSocket)?.read_next_modified_rec(handle).await {
            modified_count += 1;
        }
        
        rows.push(vec![
            db.name.clone(),
            format!("{}", modified_count),
        ]);
        
        socket.close_database(handle).await?;
    }
    
    print_table(
        &["Database", "Modified Records"],
        &rows,
    );
    
    println!("Closing conduit...");
    socket.dlp().ok_or(PilotError::DlpSocket)?.end_sync(crate::protocol::dlp::DlpEndStatus::Normal).await?;
    
    Ok(())
}
