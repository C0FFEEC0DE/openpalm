//! Protocol layer for openpalm
//!
//! This module implements the protocol stack used for communication
//! with Palm OS devices.

mod socket;
pub mod dlp;
pub mod slp;
pub mod padp;
pub mod net;
pub mod syspkt;

pub use socket::PilotSocket;
pub use dlp::{DlpClient, ProtocolVersion};
pub use slp::{SlpConnection, SlpPacket, SlpPacketType, SlpState};
pub use padp::{PadpConnection, PadpPacket, PadpType, PadpFlags, PadpState};
pub use net::{NetHandler, NetPacket, NetConnection, NetCommand, NetState, NetError};
pub use syspkt::{SysPkt, SysPktType, SysPktCmd, SysInfo, UserInfo, SysPktHandler};

/// Protocol levels in the stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolLevel {
    /// Device level
    Dev,
    /// Serial Link Protocol level
    Slp,
    /// PADP protocol level
    Padp,
    /// NET protocol level
    Net,
    /// System protocol level
    Sys,
    /// CMP protocol level
    Cmp,
    /// Desktop Link Protocol level
    Dlp,
    /// Socket level
    Sock,
}

impl ProtocolLevel {
    /// Get the protocol name
    pub fn name(&self) -> &'static str {
        match self {
            ProtocolLevel::Dev => "DEV",
            ProtocolLevel::Slp => "SLP",
            ProtocolLevel::Padp => "PADP",
            ProtocolLevel::Net => "NET",
            ProtocolLevel::Sys => "SYS",
            ProtocolLevel::Cmp => "CMP",
            ProtocolLevel::Dlp => "DLP",
            ProtocolLevel::Sock => "SOCK",
        }
    }
}
