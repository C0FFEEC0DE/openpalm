//! Protocol layer for openpalm
//!
//! This module implements the protocol stack used for communication
//! with Palm OS devices.

pub mod dlp;
pub mod net;
pub mod padp;
pub mod slp;
mod socket;
pub mod syspkt;

pub use dlp::{DlpArg, DlpClient, DlpRequest, DlpResponse, ProtocolVersion};
pub use net::{NetCommand, NetConnection, NetError, NetHandler, NetPacket, NetState};
pub use padp::{PadpConnection, PadpFlags, PadpPacket, PadpState, PadpType};
pub use slp::{SlpConnection, SlpPacket, SlpPacketType, SlpState};
pub use socket::{PilotSocket, TransportConnection};
pub use syspkt::{SysInfo, SysPkt, SysPktCmd, SysPktHandler, SysPktType, UserInfo};

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
