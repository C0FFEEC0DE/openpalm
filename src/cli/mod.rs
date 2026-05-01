//! OpenPalm CLI shared helpers

pub mod db;
pub mod datetime;
pub mod device;
pub mod record;
pub mod resource;
pub mod sync;
pub mod vfs;

use crate::error::Result;
use crate::PilotSocket;

/// Connect to a Palm device based on CLI arguments
pub async fn connect(port: Option<&str>, host: Option<&str>) -> Result<PilotSocket> {
    let mut socket = match (port, host) {
        #[cfg(feature = "serial")]
        (Some(p), _) => PilotSocket::serial(p),
        #[cfg(feature = "net")]
        (_, Some(h)) => PilotSocket::net(h, 14238),
        _ => PilotSocket::usb(),
    };
    socket.connect()?;
    Ok(socket)
}

/// Print simple aligned table
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let widths: Vec<usize> = headers.iter().enumerate().map(|(i, h)| {
        let max = rows.iter().map(|r| r.get(i).map(|s| s.len()).unwrap_or(0)).max().unwrap_or(0);
        h.len().max(max).max(8)
    }).collect();
    for (i, h) in headers.iter().enumerate() {
        print!("{:w$}  ", h, w = widths[i]);
    }
    println!();
    for w in &widths { print!("{:-<w$}  ", "", w = w); }
    println!();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            print!("{:w$}  ", cell, w = widths[i]);
        }
        println!();
    }
}
