pub use crate::error::ClockError;
pub use crate::{NTP_UNIX_EPOCH, parse_ntp_packet};
#[cfg(feature = "std")]
pub use crate::{NtpClient, unix_nanos_now};

pub use log::*;
pub type UnixTimestampNanos = u64;
