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

#[cfg(feature = "cli")]
pub mod cli;
pub mod clock;
pub mod prelude;

#[cfg(feature = "std")]
use std::net::{SocketAddr, UdpSocket};
#[cfg(feature = "std")]
use std::process::ExitCode;
#[cfg(feature = "std")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "std")]
use prelude::*;

#[cfg(feature = "std")]
pub struct NtpData {
    pub last_check: u64,
    pub last_time: u64,
    pub last_offset: i64,
}

#[cfg(feature = "std")]
pub struct NtpClient {
    pub server: SocketAddr,
    pub data: NtpData,
    pub time_validity: Duration,
}

#[cfg(feature = "std")]
impl NtpClient {
    pub fn new(server: &str) -> Result<Self, ClockError> {
        let server: SocketAddr = resolve_server(server)?;
        let validity = std::time::Duration::from_secs(60);
        Ok(NtpClient {
            server,
            data: NtpData {
                last_check: unix_nanos_now().saturating_sub(validity.as_nanos() as u64 * 3),
                last_time: unix_nanos_now(),
                last_offset: 0,
            },
            time_validity: std::time::Duration::from_secs(60),
        })
    }

    // Has the time been updated in the last `time_validity` period?
    pub fn time_is_valid(&self) -> bool {
        let last_check = self.data.last_check;
        let now = unix_nanos_now();
        let elapsed = now.saturating_sub(last_check);
        elapsed < self.time_validity.as_nanos() as u64
    }

    pub fn get_time(&mut self) -> Result<u64, ClockError> {
        if !self.time_is_valid() {
            return self.update();
        }

        Ok(self.data.last_time)
    }

    pub fn parse_packet(packet: &[u8], local_time: u64) -> Result<(u64, i64), ClockError> {
        parse_ntp_packet(packet, local_time).map_err(|_| ClockError::InvalidResponse)
    }

    pub fn update(&mut self) -> Result<u64, ClockError> {
        debug!("Updating...");
        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| ClockError::NetworkError)?;

        let mut request = [0u8; 48];
        request[0] = 0x1B;
        socket
            .send_to(&request, self.server)
            .map_err(|_| ClockError::NetworkError)?;

        let mut response = [0u8; 48];
        let recv_result = socket.recv_from(&mut response);

        let (len, _) = recv_result.map_err(|_| ClockError::NetworkError)?;
        let local_time = unix_nanos_now();
        self.update_from_response(&response[..len], local_time)
    }

    pub fn get_offset(&self) -> i64 {
        self.data.last_offset
    }

    pub fn update_from_response(
        &mut self,
        response: &[u8],
        local_time: u64,
    ) -> Result<u64, ClockError> {
        let (ntp_now, offset) = Self::parse_packet(response, local_time)?;

        self.data.last_check = ntp_now;
        self.data.last_time = ntp_now;
        self.data.last_offset = offset;
        debug!("Done updating NTP time: {}", ntp_now);
        Ok(ntp_now)
    }
}

pub const NTP_UNIX_EPOCH: i64 = 2_208_988_800;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NtpParseError {
    InvalidResponse,
}

pub fn parse_ntp_packet(packet: &[u8], local_time: u64) -> Result<(u64, i64), NtpParseError> {
    if packet.len() < 48 {
        return Err(NtpParseError::InvalidResponse);
    }

    let seconds = u32::from_be_bytes([packet[40], packet[41], packet[42], packet[43]]) as i64;
    let fraction = u32::from_be_bytes([packet[44], packet[45], packet[46], packet[47]]);
    let unix_seconds = seconds - NTP_UNIX_EPOCH;
    if unix_seconds < 0 {
        return Err(NtpParseError::InvalidResponse);
    }
    let nanos = ((fraction as u128 * 1_000_000_000u128) >> 32) as i128;
    let timestamp = (unix_seconds as i128)
        .checked_mul(1_000_000_000)
        .and_then(|secs| secs.checked_add(nanos))
        .ok_or(NtpParseError::InvalidResponse)?;
    if timestamp < 0 {
        return Err(NtpParseError::InvalidResponse);
    }
    let ntp_now = u64::try_from(timestamp).map_err(|_| NtpParseError::InvalidResponse)?;
    let offset = (ntp_now as i128)
        .checked_sub(local_time as i128)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or(NtpParseError::InvalidResponse)?;
    Ok((ntp_now, offset))
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

#[cfg(feature = "std")]
#[derive(Clone, Debug)]
pub enum ClockError {
    NetworkError,
    InvalidResponse,
    ConfigError(String),
    NoTimeAvailable,
    Timeout,
    Io(String),
}

#[cfg(feature = "std")]
impl std::fmt::Display for ClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClockError::NetworkError => write!(f, "Network error occurred"),
            ClockError::InvalidResponse => write!(f, "Received invalid response from NTP server"),
            ClockError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            ClockError::NoTimeAvailable => write!(f, "No valid time available"),
            ClockError::Timeout => write!(f, "Operation timed out"),
            ClockError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for ClockError {
    fn from(err: std::io::Error) -> Self {
        ClockError::Io(err.to_string())
    }
}

#[cfg(feature = "std")]
impl From<std::net::AddrParseError> for ClockError {
    fn from(err: std::net::AddrParseError) -> Self {
        ClockError::ConfigError(err.to_string())
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
            ClockError::Io(_) => ExitCode::from(6),
        }
    }
}
