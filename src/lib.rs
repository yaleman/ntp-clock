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

pub mod cli;
pub mod clock;
pub mod prelude;

use std::process::ExitCode;
use std::{net::SocketAddr, sync::Arc, time::Duration};

use hickory_resolver::Resolver;
use prelude::*;
use time::Duration as TimeDuration;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::timeout;

pub struct NtpData {
    pub last_check: OffsetDateTime,
    pub last_time: OffsetDateTime,
    pub last_offset: TimeDuration,
}

pub struct NtpClient {
    pub server: SocketAddr,
    pub data: Arc<RwLock<NtpData>>,
    pub time_validity: std::time::Duration,
}

impl NtpClient {
    pub async fn new(server: &str) -> Result<Self, ClockError> {
        let server: SocketAddr = resolve_server(server).await?;
        let validity = std::time::Duration::from_secs(60);
        Ok(NtpClient {
            server,
            data: Arc::new(RwLock::new(NtpData {
                last_check: OffsetDateTime::now_utc() - (validity * 3),
                last_time: OffsetDateTime::now_utc(),
                last_offset: TimeDuration::ZERO,
            })),
            time_validity: std::time::Duration::from_secs(60),
        })
    }

    // Has the time been updated in the last `time_validity` period?
    pub async fn time_is_valid(&self) -> bool {
        let data = self.data.read().await;
        let last_check = data.last_check;
        let now = OffsetDateTime::now_utc();
        let elapsed = now - last_check;
        elapsed.whole_seconds() < self.time_validity.as_secs() as i64
    }

    pub async fn get_time(&self) -> Result<OffsetDateTime, ClockError> {
        if !self.time_is_valid().await {
            return self.update().await;
        }

        let data = self.data.read().await;
        Ok(data.last_time)
    }

    fn parse_packet(packet: &[u8]) -> Result<OffsetDateTime, ClockError> {
        if packet.len() < 48 {
            return Err(ClockError::InvalidResponse);
        }

        let seconds = u32::from_be_bytes([packet[40], packet[41], packet[42], packet[43]]) as i64;
        let fraction = u32::from_be_bytes([packet[44], packet[45], packet[46], packet[47]]);
        let unix_seconds = seconds - 2_208_988_800i64;
        let nanos = ((fraction as u128 * 1_000_000_000u128) >> 32) as i128;
        let timestamp = (unix_seconds as i128)
            .checked_mul(1_000_000_000)
            .and_then(|secs| secs.checked_add(nanos))
            .ok_or(ClockError::InvalidResponse)?;
        let ntp_now = OffsetDateTime::from_unix_timestamp_nanos(timestamp)
            .map_err(|_| ClockError::InvalidResponse)?;
        Ok(ntp_now)
    }

    pub async fn update(&self) -> Result<OffsetDateTime, ClockError> {
        debug!("Updating...");
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|_| ClockError::NetworkError)?;

        let mut request = [0u8; 48];
        request[0] = 0x1B;
        socket
            .send_to(&request, self.server)
            .await
            .map_err(|_| ClockError::NetworkError)?;

        let mut response = [0u8; 48];
        let recv_result = timeout(Duration::from_secs(5), socket.recv_from(&mut response))
            .await
            .map_err(|_| ClockError::Timeout)?;

        let (len, _) = recv_result.map_err(|_| ClockError::NetworkError)?;
        if len < 48 {
            return Err(ClockError::InvalidResponse);
        }

        let ntp_now = Self::parse_packet(&response)?;
        let local_time = OffsetDateTime::now_utc();

        let mut data = self.data.write().await;
        data.last_check = ntp_now;
        data.last_time = ntp_now;
        data.last_offset = ntp_now - local_time;
        debug!("Done updating NTP time: {}", ntp_now);
        Ok(ntp_now)
    }

    pub async fn get_offset(&self) -> TimeDuration {
        let data = self.data.read().await;
        data.last_offset
    }
}

async fn resolve_server(server: &str) -> Result<SocketAddr, ClockError> {
    if let Ok(addr) = server.parse::<SocketAddr>() {
        return Ok(addr);
    }

    let (host, port) = if let Some((host, port)) = server.rsplit_once(':') {
        (
            host,
            port.parse::<u16>()
                .map_err(|_| ClockError::ConfigError(server.to_string()))?,
        )
    } else {
        (server, 123)
    };

    let resolver = Resolver::builder_tokio()?.build();
    let lookup = resolver
        .lookup_ip(host)
        .await
        .map_err(|_| ClockError::NetworkError)?;
    let ip = lookup.iter().next().ok_or(ClockError::InvalidResponse)?;
    Ok(SocketAddr::new(ip, port))
}

#[derive(Clone, Debug)]
pub enum ClockError {
    NetworkError,
    InvalidResponse,
    ConfigError(String),
    NoTimeAvailable,
    Timeout,
}

impl std::fmt::Display for ClockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClockError::NetworkError => write!(f, "Network error occurred"),
            ClockError::InvalidResponse => write!(f, "Received invalid response from NTP server"),
            ClockError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            ClockError::NoTimeAvailable => write!(f, "No valid time available"),
            ClockError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl From<std::net::AddrParseError> for ClockError {
    fn from(err: std::net::AddrParseError) -> Self {
        ClockError::ConfigError(err.to_string())
    }
}

impl From<hickory_resolver::ResolveError> for ClockError {
    fn from(err: hickory_resolver::ResolveError) -> Self {
        ClockError::ConfigError(format!("DNS resolution failed: {err}"))
    }
}

impl From<ClockError> for ExitCode {
    fn from(value: ClockError) -> Self {
        match value {
            ClockError::NetworkError => ExitCode::from(1),
            ClockError::InvalidResponse => ExitCode::from(2),
            ClockError::ConfigError(_) => ExitCode::from(3),
            ClockError::NoTimeAvailable => ExitCode::from(4),
            ClockError::Timeout => ExitCode::from(5),
        }
    }
}
