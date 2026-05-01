//! VFS commands

use crate::error::Result;
use crate::PilotSocket;
use crate::cli::print_table;

/// List VFS volumes
pub async fn volumes(socket: &mut PilotSocket) -> Result<()> {
    let vols = socket.dlp().unwrap().vfs_volume_enumerate().await?;
    
    if vols.is_empty() {
        println!("No VFS volumes found.");
        return Ok(());
    }
    
    let mut rows = Vec::new();
    for vol_ref in &vols {
        match socket.dlp().unwrap().vfs_volume_info(*vol_ref).await {
            Ok(info) => {
                rows.push(vec![
                    format!("{}", vol_ref.value()),
                    info.label.clone(),
                    format!("{}", info.fs_type),
                    format!("{}", info.media_type),
                ]);
            }
            Err(e) => {
                rows.push(vec![
                    format!("{}", vol_ref.value()),
                    format!("(error: {})", e),
                    String::new(),
                    String::new(),
                ]);
            }
        }
    }
    
    print_table(
        &["Vol Ref", "Label", "FS Type", "Media Type"],
        &rows,
    );
    Ok(())
}
