pub mod cli;
pub mod prelude;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use hickory_resolver::Resolver;
use prelude::*;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::timeout;

pub struct NtpData {
    pub last_check: OffsetDateTime,
    pub last_time: OffsetDateTime,
}

pub struct NtpClient {
    pub server: SocketAddr,
    pub data: Arc<RwLock<NtpData>>,
    pub time_validity: std::time::Duration,
}

impl NtpClient {
    pub async fn new(server: &str) -> Result<Self, ClockError> {
        let server = resolve_server(server).await?;
        Ok(NtpClient {
            server,
            data: Arc::new(RwLock::new(NtpData {
                last_check: OffsetDateTime::now_utc(),
                last_time: OffsetDateTime::now_utc(),
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

    pub async fn update(&self) -> Result<OffsetDateTime, ClockError> {
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
            .map_err(|_| ClockError::NetworkError)?;

        let (len, _) = recv_result.map_err(|_| ClockError::NetworkError)?;
        if len < 48 {
            return Err(ClockError::InvalidResponse);
        }

        let seconds =
            u32::from_be_bytes([response[40], response[41], response[42], response[43]]) as i64;
        let fraction = u32::from_be_bytes([response[44], response[45], response[46], response[47]]);
        let unix_seconds = seconds - 2_208_988_800i64;
        let nanos = ((fraction as u128 * 1_000_000_000u128) >> 32) as i128;
        let timestamp = (unix_seconds as i128)
            .checked_mul(1_000_000_000)
            .and_then(|secs| secs.checked_add(nanos))
            .ok_or(ClockError::InvalidResponse)?;
        let now = OffsetDateTime::from_unix_timestamp_nanos(timestamp)
            .map_err(|_| ClockError::InvalidResponse)?;

        let mut data = self.data.write().await;
        data.last_check = now;
        data.last_time = now;
        Ok(now)
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
