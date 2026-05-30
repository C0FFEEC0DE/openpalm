# OpenPalm

Talk to your Palm OS device from the command line.

## Quick Start

```bash
# Build
cargo build --release --bin op

# Connect and show device info
op --port /dev/ttyUSB0 info

# List databases
op --port /dev/ttyUSB0 db list

# Export calendar to a file
op --port /dev/ttyUSB0 db export --name DatebookDB --output datebook.pdb
```

## Library Example

```rust
use openpalm::PilotSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = PilotSocket::serial("/dev/ttyUSB0");
    socket.connect()?;

    let info = socket.read_sys_info().await?;
    println!("ROM: {}.{}", info.rom_version_major, info.rom_version_minor);

    let dbs = socket.list_databases().await?;
    for db in dbs {
        println!("{}", db.name);
    }

    Ok(())
}
```

## System Dependencies

**Fedora:** `sudo dnf install libusb1-devel pkg-config`

**Ubuntu/Debian:** `sudo apt install libusb-1.0-0-dev pkg-config`

**macOS:** libusb comes pre-installed.

**Windows:** Install the WinUSB driver with [Zadig](https://zadig.akeo.ie/).

## License

GPL-2.0 or later
