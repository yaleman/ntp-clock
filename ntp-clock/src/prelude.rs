#[cfg(feature = "std")]
pub use crate::{ClockError, NtpClient, unix_nanos_now};
pub use crate::{NTP_UNIX_EPOCH, parse_ntp_packet};

pub use log::*;
pub type UnixTimestampNanos = u64;
