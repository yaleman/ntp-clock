#[cfg(feature = "std")]
pub use crate::{ClockError, NtpClient, unix_nanos_now};
pub use crate::{NTP_UNIX_EPOCH, NtpParseError, parse_ntp_packet};

#[cfg(feature = "cli")]
pub use clap::Parser;

pub use log::*;
pub type UnixTimestampNanos = u64;
