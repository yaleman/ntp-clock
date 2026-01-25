#![deny(warnings)]
#![warn(unused_extern_crates)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unreachable)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::needless_pass_by_value)]
#![deny(clippy::trivially_copy_pass_by_ref)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(target_family = "unix", target_family = "windows"))]
pub mod cli;

pub mod clock;
pub mod constants;
pub mod packets;
pub mod prelude;

#[cfg(feature = "std")]
use std::net::{SocketAddr, UdpSocket};
#[cfg(feature = "std")]
use std::process::ExitCode;
#[cfg(feature = "std")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use packed_struct::PackedStructSlice;
#[cfg(feature = "std")]
use prelude::*;

use crate::constants::NTP_MIN_PACKET_LEN;
use crate::packets::NtpResponse;

#[cfg(feature = "std")]
pub struct NtpData {
    /// When we last updated this
    pub last_check: u64,

    /// The last NTP response we received
    pub last_response: Option<NtpResponse>,
}

#[cfg(feature = "std")]
impl NtpData {
    pub fn offset(&self) -> Option<i64> {
        self.last_response
            .as_ref()
            .map(|resp| resp.offset_from_local(self.last_check))
    }
}

#[cfg(feature = "std")]
pub struct NtpClient {
    pub server: SocketAddr,
    pub time_validity: Duration,
    pub last_response: Option<NtpResponse>,
}

#[cfg(feature = "std")]
impl NtpClient {
    pub fn new(server: &str) -> Result<Self, ClockError> {
        let server: SocketAddr = resolve_server(server)?;
        Ok(NtpClient {
            server,

            time_validity: std::time::Duration::from_secs(60),
            last_response: None,
        })
    }

    // Has the time been updated in the last `time_validity` period?
    pub fn time_is_valid(&self) -> bool {
        match self.last_response.as_ref() {
            None => false,
            Some(response) => {
                let now = unix_nanos_now();
                let elapsed = now.saturating_sub(response.ref_time);
                elapsed < self.time_validity.as_nanos() as u64
            }
        }
    }

    pub fn get_time(&mut self) -> Result<u64, ClockError> {
        if !self.time_is_valid() {
            return self.update();
        }
        match self.last_response.as_ref() {
            Some(response) => Ok(response.ref_time),
            None => Err(ClockError::NoTimeAvailable),
        }
    }

    pub fn update(&mut self) -> Result<u64, ClockError> {
        use packed_struct::PackedStruct;

        use crate::{constants::NTP_MIN_PACKET_LEN, packets::NtpRequestPacket};

        debug!("Updating...");
        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| ClockError::NetworkError)?;

        let request: [u8; 48] = NtpRequestPacket::default().pack().map_err(|err| {
            #[cfg(feature = "std")]
            error!("Failed to pack NTP request packet: {:?}", err);
            ClockError::Io
        })?;
        socket
            .send_to(&request, self.server)
            .map_err(|_| ClockError::NetworkError)?;

        let mut response = [0u8; NTP_MIN_PACKET_LEN];
        let (len, _) = socket
            .recv_from(&mut response)
            .map_err(|_| ClockError::NetworkError)?;
        let local_time = unix_nanos_now();
        self.update_from_response(&response[..len], local_time)
    }

    pub fn update_from_response(
        &mut self,
        response: &[u8],
        local_time: u64,
    ) -> Result<u64, ClockError> {
        let response = parse_ntp_packet(response, local_time)?;
        self.last_response = Some(response.clone());

        debug!("Done updating NTP time: {}", response.transmit_time);
        Ok(response.transmit_time)
    }
}

pub const NTP_UNIX_EPOCH: i64 = 2_208_988_800;

/// Returns the NTP time and offset in nanoseconds.
pub fn parse_ntp_packet(packet: &[u8], _local_time: u64) -> Result<NtpResponse, ClockError> {
    if packet.len() < NTP_MIN_PACKET_LEN {
        return Err(ClockError::PacketTooShort);
    }
    #[cfg(all(any(debug_assertions, test), feature = "std"))]
    {
        eprintln!("NTP packet length: {}", packet.len());
        eprintln!(
            "NTP Packet: {:?}",
            packet
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
        );
    }
    let mut result = [0u8; 60];
    result[..packet.len()].copy_from_slice(packet);
    let res = NtpResponse::unpack_from_slice(&result).map_err(|_| ClockError::InvalidResponse)?;

    if res.version < 1 || res.version > 4 {
        return Err(ClockError::InvalidVersion);
    }

    Ok(res)
}

#[cfg(feature = "std")]
pub fn unix_nanos_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(feature = "std")]
fn resolve_server(server: &str) -> Result<SocketAddr, ClockError> {
    use std::net::ToSocketAddrs;
    let addrs: Vec<SocketAddr> = (server, 123).to_socket_addrs()?.collect();
    if let Some(addr) = addrs.first() {
        Ok(*addr)
    } else {
        Err(ClockError::ConfigError(format!(
            "Could not resolve NTP server address: {}",
            server
        )))
    }
}

#[derive(Clone, Debug)]
pub enum ClockError {
    NetworkError,
    Io,
    InvalidResponse,
    #[cfg(feature = "std")]
    ConfigError(std::string::String),
    NoTimeAvailable,
    Timeout,
    PacketTooShort,
    InvalidIdentifier,
    InvalidVersion,
}

#[cfg(feature = "std")]
impl std::fmt::Display for ClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClockError::NetworkError => write!(f, "Network error occurred"),
            #[cfg(feature = "std")]
            ClockError::InvalidResponse => {
                write!(f, "Received invalid response from NTP server")
            }
            ClockError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            ClockError::NoTimeAvailable => write!(f, "No valid time available"),
            ClockError::Timeout => write!(f, "Operation timed out"),
            ClockError::Io => write!(f, "I/O error"),
            ClockError::PacketTooShort => write!(f, "NTP packet too short"),
            ClockError::InvalidIdentifier => write!(f, "Invalid NTP identifier"),
            ClockError::InvalidVersion => write!(f, "Invalid NTP version"),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for ClockError {
    fn from(err: std::io::Error) -> Self {
        error!("IoError: {}", err);
        ClockError::Io
    }
}

#[cfg(feature = "std")]
impl From<std::net::AddrParseError> for ClockError {
    fn from(err: std::net::AddrParseError) -> Self {
        ClockError::ConfigError(format!("Failed to parse address: {}", err))
    }
}

#[cfg(feature = "std")]
impl From<ClockError> for ExitCode {
    fn from(value: ClockError) -> Self {
        match value {
            ClockError::NetworkError => ExitCode::from(1),
            ClockError::InvalidResponse => ExitCode::from(2),
            ClockError::ConfigError(_) => ExitCode::from(3),
            ClockError::NoTimeAvailable => ExitCode::from(4),
            ClockError::Timeout => ExitCode::from(5),
            ClockError::Io => ExitCode::from(6),
            ClockError::PacketTooShort => ExitCode::from(7),
            _ => ExitCode::from(1),
        }
    }
}
