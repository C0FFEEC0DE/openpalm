//! OpenPalm CLI shared helpers

pub mod datetime;
pub mod db;
pub mod device;
pub mod record;
pub mod resource;
pub mod sync;
pub mod vfs;

use crate::error::Result;
use crate::PilotSocket;

/// Connect to a Palm device based on CLI arguments
#[allow(unused_variables)]
pub async fn connect(port: Option<&str>, host: Option<&str>) -> Result<PilotSocket> {
    #[cfg(feature = "serial")]
    if let Some(p) = port {
        let mut socket = PilotSocket::serial(p);
        socket.connect()?;
        return Ok(socket);
    }

    #[cfg(feature = "net")]
    if let Some(h) = host {
        let mut socket = PilotSocket::net(h, 14238);
        socket.connect()?;
        return Ok(socket);
    }

    #[cfg(feature = "usb")]
    {
        let mut socket = PilotSocket::usb();
        socket.connect()?;
        Ok(socket)
    }

    #[cfg(not(any(feature = "serial", feature = "usb", feature = "net")))]
    return Err(crate::error::PilotError::SockInvalid);
}

/// Execute a command with a connected socket, ensuring disconnect is always called
pub async fn with_connection<F>(port: Option<&str>, host: Option<&str>, f: F) -> Result<()>
where
    F: for<'a> FnOnce(
        &'a mut PilotSocket,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>>,
{
    let mut socket = connect(port, host).await?;
    let result = f(&mut socket).await;
    let _ = socket.disconnect();
    result
}

/// Print simple aligned table
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let max = rows
                .iter()
                .map(|r| r.get(i).map(|s| s.len()).unwrap_or(0))
                .max()
                .unwrap_or(0);
            h.len().max(max).max(8)
        })
        .collect();
    for (i, h) in headers.iter().enumerate() {
        print!("{:w$}  ", h, w = widths[i]);
    }
    println!();
    for w in &widths {
        print!("{:-<w$}  ", "", w = w);
    }
    println!();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            print!("{:w$}  ", cell, w = widths[i]);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_table_empty() {
        print_table(&["Name", "Value"], &[]);
    }

    #[test]
    fn test_print_table_with_data() {
        print_table(
            &["Name", "Value"],
            &[vec!["test".to_string(), "123".to_string()]],
        );
    }

    #[test]
    fn test_print_table_multiple_rows() {
        print_table(
            &["A", "B", "C"],
            &[
                vec!["1".to_string(), "2".to_string(), "3".to_string()],
                vec!["longer".to_string(), "x".to_string(), "y".to_string()],
            ],
        );
    }
}
