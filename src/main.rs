//! OpenPalm CLI — minimal stub binary
//!
//! This is the `palm` command-line interface for the openpalm library.

use clap::{Parser, Subcommand};

// Demonstrate access to the openpalm library public API.
// These imports would be used by the real implementation.
#[allow(unused_imports)]
use openpalm::{
    PilotError, Result as OpenPalmResult,
    DatabaseInfo, Record,
    SyncHandler, SyncDirection, SyncStats,
    DlpClient, ProtocolVersion,
};

/// OpenPalm CLI — communicate with Palm OS devices
#[derive(Parser)]
#[command(name = "palm")]
#[command(about = "CLI for Palm OS device communication")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Synchronize with a Palm device
    Sync {
        /// Serial port path (e.g., /dev/ttyUSB0)
        #[arg(short, long)]
        port: Option<String>,
        /// Network host for network sync
        #[arg(short, long)]
        host: Option<String>,
    },
    /// List databases on the Palm device
    List,
    /// Read system and user info from the device
    Info,
    /// Dump a database from the device
    Dump {
        /// Name of the database to dump
        #[arg(short, long)]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync { port, host } => {
            if let Some(p) = port {
                println!("Sync via serial port: {}", p);
            }
            if let Some(h) = host {
                println!("Sync via network host: {}", h);
            }
            println!("Command 'sync' is not yet implemented.");
        }
        Commands::List => {
            println!("Command 'list' is not yet implemented.");
        }
        Commands::Info => {
            println!("Command 'info' is not yet implemented.");
        }
        Commands::Dump { name } => {
            println!("Dumping database: {}", name);
            println!("Command 'dump' is not yet implemented.");
        }
    }

    Ok(())
}
