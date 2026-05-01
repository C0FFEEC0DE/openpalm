//! OpenPalm CLI

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "palm")]
#[command(about = "CLI for Palm OS device communication")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Serial port path (e.g. /dev/ttyUSB0)
    #[arg(short, long)]
    port: Option<String>,
    /// Network host for network sync
    #[arg(short = 'H', long)]
    host: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show device information
    Info,
    /// Database operations
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },
    /// Record operations
    Record {
        #[command(subcommand)]
        command: RecordCommands,
    },
    /// Resource operations
    Resource {
        #[command(subcommand)]
        command: ResourceCommands,
    },
    /// VFS operations
    Vfs {
        #[command(subcommand)]
        command: VfsCommands,
    },
    /// Sync with device
    Sync,
    /// Device datetime
    Datetime {
        #[command(subcommand)]
        command: DatetimeCommands,
    },
    /// Start network sync server
    #[cfg(feature = "net")]
    Server {
        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
        /// Listen port
        #[arg(short, long, default_value_t = 14238)]
        port: u16,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// List all databases
    List,
    /// Show database info
    Info {
        /// Database name
        name: String,
    },
    /// Dump a database
    Dump {
        /// Database name
        name: String,
    },
    /// Create a new database
    Create {
        /// Database name
        name: String,
        /// Creator code (4 chars)
        #[arg(short, long)]
        creator: String,
        /// Database type (4 chars)
        #[arg(short, long)]
        db_type: String,
    },
    /// Delete a database
    Delete {
        /// Database name
        name: String,
    },
    /// Export a database to a PDB file
    Export {
        /// Database name
        name: String,
        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

#[derive(Subcommand)]
enum RecordCommands {
    /// List records in a database
    List {
        /// Database name
        db: String,
    },
    /// Read a record by index
    Read {
        /// Database name
        db: String,
        /// Record index
        index: u32,
    },
}

#[derive(Subcommand)]
enum ResourceCommands {
    /// List resources in a database
    List {
        /// Database name
        db: String,
    },
}

#[derive(Subcommand)]
enum VfsCommands {
    /// List VFS volumes
    Volumes,
}

#[derive(Subcommand)]
enum DatetimeCommands {
    /// Show device datetime
    Show,
    /// Set device datetime to current system time
    Set,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        #[cfg(feature = "net")]
        Commands::Server { bind, port } => {
            println!("Starting server on {}:{}", bind, port);
            let mut socket = openpalm::PilotSocket::net_listen(
                &bind,
                port,
            )?;
            println!("Waiting for connection...");
            socket.accept()?;
            println!("Client connected!");
            openpalm::cli::device::info(&mut socket).await?;
            socket.disconnect()?;
        }
        Commands::Sync => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::sync::sync_device(&mut socket).await?;
            socket.disconnect()?;
        }
        Commands::Info => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::device::info(&mut socket).await?;
            socket.disconnect()?;
        }
        Commands::Db { command: DbCommands::List } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::db::list(&mut socket).await?;
            socket.disconnect()?;
        }
        Commands::Db { command: DbCommands::Info { name } } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::db::info(&mut socket, &name).await?;
            socket.disconnect()?;
        }
        Commands::Db { command: DbCommands::Dump { name } } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::db::dump(&mut socket, &name).await?;
            socket.disconnect()?;
        }
        Commands::Db { command: DbCommands::Create { name, creator, db_type } } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::db::create(&mut socket, &name, &creator, &db_type).await?;
            socket.disconnect()?;
        }
        Commands::Db { command: DbCommands::Delete { name } } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::db::delete(&mut socket, &name).await?;
            socket.disconnect()?;
        }
        Commands::Db { command: DbCommands::Export { name, output } } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::db::export(&mut socket, &name, &output).await?;
            socket.disconnect()?;
        }
        Commands::Record { command: RecordCommands::List { db } } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::record::list(&mut socket, &db).await?;
            socket.disconnect()?;
        }
        Commands::Record { command: RecordCommands::Read { db, index } } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::record::read(&mut socket, &db, index).await?;
            socket.disconnect()?;
        }
        Commands::Resource { command: ResourceCommands::List { db } } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::resource::list(&mut socket, &db).await?;
            socket.disconnect()?;
        }
        Commands::Vfs { command: VfsCommands::Volumes } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::vfs::volumes(&mut socket).await?;
            socket.disconnect()?;
        }
        Commands::Datetime { command: DatetimeCommands::Show } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::datetime::show(&mut socket).await?;
            socket.disconnect()?;
        }
        Commands::Datetime { command: DatetimeCommands::Set } => {
            let mut socket = openpalm::cli::connect(
                cli.port.as_deref(),
                cli.host.as_deref(),
            ).await?;
            openpalm::cli::datetime::set_now(&mut socket).await?;
            socket.disconnect()?;
        }
    }

    Ok(())
}
