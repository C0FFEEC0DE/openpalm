//! Database commands

use crate::cli::print_table;
use crate::error::{PilotError, Result};
use crate::types::FourCharCode;
use crate::PilotSocket;
use std::fs::File;
use std::io::Write;

/// List all databases on the device
pub async fn list(socket: &mut PilotSocket) -> Result<()> {
    let dbs = socket.list_databases().await?;
    let rows: Vec<Vec<String>> = dbs
        .iter()
        .map(|db| {
            vec![
                db.name.clone(),
                db.creator.to_string(),
                db.db_type.to_string(),
                format!("{}", db.num_records),
                format!("{}", db.total_bytes),
            ]
        })
        .collect();
    print_table(&["Name", "Creator", "Type", "Records", "Size"], &rows);
    Ok(())
}

/// Show detailed info for a database
pub async fn info(socket: &mut PilotSocket, name: &str) -> Result<()> {
    let dbs = socket.list_databases().await?;
    let db = dbs.iter().find(|d| d.name == name);

    match db {
        Some(db) => {
            println!("Database:    {}", db.name);
            println!("Creator:     {}", db.creator);
            println!("Type:        {}", db.db_type);
            println!("Card:        {}", db.card_no);
            println!("Records:     {}", db.num_records);
            println!("Total bytes: {}", db.total_bytes);
            println!("Data bytes:  {}", db.data_bytes);
            println!("Flags:       {:?}", db.flags);
            println!("Created:     {:?}", db.created);
            println!("Modified:    {:?}", db.modified);
        }
        None => {
            println!("Database '{}' not found.", name);
        }
    }

    Ok(())
}

/// Dump a database to stdout
pub async fn dump(socket: &mut PilotSocket, name: &str) -> Result<()> {
    use crate::protocol::dlp::DlpOpenMode;
    let handle = socket.open_database(name, DlpOpenMode::Read).await?;
    let db_info = socket
        .dlp()
        .ok_or(PilotError::DlpSocket)?
        .read_open_db_info(0, handle)
        .await?;

    println!(
        "Dumping database: {} ({} records)",
        name, db_info.1.num_records
    );

    for i in 0..db_info.1.num_records {
        match socket.read_record(handle, i).await {
            Ok(record) => {
                println!("\n--- Record {} ---", i);
                println!("  ID:     0x{:08X}", record.id);
                println!("  Attrs:  {:?}", record.attributes);
                println!("  Size:   {} bytes", record.data.len());
                if let Some(s) = record.data_as_str() {
                    println!("  Text:   {}", s);
                }
            }
            Err(e) => {
                println!("  Error reading record {}: {}", i, e);
            }
        }
    }

    socket.close_database(handle).await?;
    Ok(())
}

/// Create a new database
pub async fn create(
    socket: &mut PilotSocket,
    name: &str,
    creator: &str,
    db_type: &str,
) -> Result<()> {
    if creator.len() != 4 || db_type.len() != 4 {
        return Err(PilotError::InvalidArgument);
    }
    let creator = FourCharCode::from_str(creator);
    let db_type = FourCharCode::from_str(db_type);

    let card = socket
        .dlp()
        .ok_or(PilotError::DlpSocket)?
        .create_db(
            creator,
            db_type,
            0,
            crate::types::DatabaseFlags::empty(),
            0,
            name,
        )
        .await?;
    println!("Created database '{}' on card {}", name, card);
    Ok(())
}

/// Delete a database
pub async fn delete(socket: &mut PilotSocket, name: &str) -> Result<()> {
    socket
        .dlp()
        .ok_or(PilotError::DlpSocket)?
        .delete_db(0, name)
        .await?;
    println!("Deleted database '{}'.", name);
    Ok(())
}

/// Export a database to a PDB file
pub async fn export(socket: &mut PilotSocket, name: &str, output: &str) -> Result<()> {
    use crate::protocol::dlp::DlpOpenMode;
    let handle = socket.open_database(name, DlpOpenMode::Read).await?;
    let db_info = socket
        .dlp()
        .ok_or(PilotError::DlpSocket)?
        .read_open_db_info(0, handle)
        .await?;
    let num_records = db_info.1.num_records as usize;

    let mut file =
        File::create(output).map_err(|e| crate::error::PilotError::FileError(e.to_string()))?;

    // Build header
    let mut header = crate::database::DatabaseHeader::default();
    {
        let mut name_bytes = [0u8; 32];
        let name_str = name.as_bytes();
        let len = name_str.len().min(31);
        name_bytes[..len].copy_from_slice(&name_str[..len]);
        header.name = name_bytes;
    }
    header.flags = db_info.1.flags.bits();
    header.version = 0;
    header.created = db_info.1.created.to_palm();
    header.modified = db_info.1.modified.to_palm();
    header.backup = 0;
    header.mod_num = db_info.1.mod_num;
    header.app_info_id = 0;
    header.sort_info_id = 0;
    header.db_type = db_info.1.db_type.to_u32();
    header.creator = db_info.1.creator.to_u32();
    header.unique_id_seed = 0;
    header.next_rec_list_id = 0;
    header.num_records = db_info.1.num_records;
    header.unique_record_seed = 0;

    file.write_all(&header.to_bytes())
        .map_err(|e| crate::error::PilotError::FileError(e.to_string()))?;

    // Read all records
    let mut records = Vec::with_capacity(num_records);
    for i in 0..num_records as u32 {
        match socket.read_record(handle, i).await {
            Ok(record) => records.push(record),
            Err(e) => {
                eprintln!("Warning: failed to read record {}: {}", i, e);
                records.push(crate::database::Record::new(0, vec![]));
            }
        }
    }

    // Write record entries (offset, attributes, uniqueID)
    let header_size = 86u32;
    let entry_size = 8u32;
    let mut current_offset = header_size + entry_size * num_records as u32;
    for record in &records {
        file.write_all(&current_offset.to_be_bytes())
            .map_err(|e| crate::error::PilotError::FileError(e.to_string()))?;
        file.write_all(&[record.attributes.bits()])
            .map_err(|e| crate::error::PilotError::FileError(e.to_string()))?;
        let id_bytes = record.id.to_be_bytes();
        file.write_all(&id_bytes[1..4])
            .map_err(|e| crate::error::PilotError::FileError(e.to_string()))?;
        current_offset += record.data.len() as u32;
    }

    // Write record data
    for record in &records {
        if !record.data.is_empty() {
            file.write_all(&record.data)
                .map_err(|e| crate::error::PilotError::FileError(e.to_string()))?;
        }
    }

    println!(
        "Exported database '{}' to '{}' ({} records)",
        name, output, num_records
    );

    socket.close_database(handle).await?;
    Ok(())
}
