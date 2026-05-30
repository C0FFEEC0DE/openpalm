//! Device information commands

use crate::error::Result;
use crate::PilotSocket;

/// Show device system and user info
pub async fn info(socket: &mut PilotSocket) -> Result<()> {
    let sys = socket.read_sys_info().await?;
    let user = socket.read_user_info().await?;
    let rom_major = (sys.rom_version >> 24) & 0xFF;
    let rom_minor = (sys.rom_version >> 16) & 0xFF;
    println!("Product ID:  {}", sys.prod_id);
    println!(
        "ROM Version: {}.{} (0x{:08X})",
        rom_major, rom_minor, sys.rom_version
    );
    println!("DLP Version: {}.{}", sys.dlp_major, sys.dlp_minor);
    println!("Username:    {}", user.username);
    println!("User ID:     {}", user.user_id);
    println!("Viewer ID:   {}", user.viewer_id);
    Ok(())
}
