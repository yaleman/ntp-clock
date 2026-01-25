#[cfg(feature = "std")]
use crate::prelude::*;

#[cfg(feature = "std")]
use std::process::ExitCode;

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
