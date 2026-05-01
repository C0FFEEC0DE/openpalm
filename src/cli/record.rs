//! Record commands

use crate::error::{PilotError, Result};
use crate::PilotSocket;
use crate::cli::print_table;
use crate::protocol::dlp::DlpOpenMode;

/// List records in a database
pub async fn list(socket: &mut PilotSocket, db_name: &str) -> Result<()> {
    let handle = socket.open_database(db_name, DlpOpenMode::Read).await?;
    let ids = socket.dlp().ok_or(PilotError::DlpSocket)?.read_record_id_list(handle, false, 0, u16::MAX as u32).await?;
    
    let rows: Vec<Vec<String>> = ids.iter().enumerate().map(|(idx, id)| {
        vec![format!("{}", idx), format!("0x{:08X}", id)]
    }).collect();
    
    print_table(
        &["Index", "Record ID"],
        &rows,
    );
    
    socket.close_database(handle).await?;
    Ok(())
}

/// Read a record by index
pub async fn read(socket: &mut PilotSocket, db_name: &str, index: u32) -> Result<()> {
    let handle = socket.open_database(db_name, DlpOpenMode::Read).await?;
    let record = socket.read_record(handle, index).await?;
    
    println!("Record ID:   0x{:08X}", record.id);
    println!("Index:       {}", record.index);
    println!("Category:    {}", record.category);
    println!("Attributes:  {:?}", record.attributes);
    println!("Size:        {} bytes", record.data.len());
    
    if let Some(s) = record.data_as_str() {
        println!("Data (text): {}", s);
    } else {
        println!("Data (hex):  {}", crate::utils::bytes_to_hex(&record.data));
    }
    
    socket.close_database(handle).await?;
    Ok(())
}
