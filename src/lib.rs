pub mod cli;
pub mod prelude;

use std::{net::SocketAddr, sync::Arc};

use prelude::*;
use tokio::sync::RwLock;

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
    pub fn new(server: &str) -> Result<Self, ClockError> {
        let server: SocketAddr = server.parse().map_err(ClockError::from)?;
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
        let data = self.data.blocking_read();
        let last_check = data.last_check;
        let now = OffsetDateTime::now_utc();
        let elapsed = now - last_check;
        elapsed.whole_seconds() < self.time_validity.as_secs() as i64
    }

    pub async fn get_time(&self) -> Result<OffsetDateTime, ClockError> {
        // Placeholder for NTP time fetching logic
        let data = self.data.read().await;
        let last_time = data.last_time;
        Ok(last_time)
    }

    pub async fn update(&mut self) -> Result<OffsetDateTime, ClockError> {
        // Placeholder for NTP time fetching logic
        let mut data = self.data.write().await;
        let now = OffsetDateTime::now_utc();
        data.last_check = now;
        data.last_time = now;
        Ok(now)
    }
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
