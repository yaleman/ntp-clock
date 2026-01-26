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

#[cfg(feature = "std")]
pub mod cli;

pub mod clock;
pub mod constants;
pub mod error;
pub mod packets;
pub mod prelude;

#[cfg(feature = "std")]
use std::net::{SocketAddr, UdpSocket};
#[cfg(feature = "std")]
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::ClockError;
use packed_struct::PackedStructSlice;
#[cfg(feature = "std")]
use prelude::*;

use crate::constants::NTP_MIN_PACKET_LEN;
use crate::packets::NtpPacket;

#[cfg(feature = "std")]
pub struct NtpData {
    /// When we last updated this
    pub last_check: u64,

    /// The last NTP response we received
    pub last_response: Option<NtpPacket>,
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
    pub last_response: Option<NtpPacket>,
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
                let elapsed = now.saturating_sub(response.transmit_time);
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

        use crate::{constants::NTP_MIN_PACKET_LEN, packets::NtpPacket};

        debug!("Updating...");
        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| ClockError::NetworkError)?;

        let transmit_time = unix_nanos_now();
        let ntp_transmit_time = unix_nanos_to_ntp_timestamp(transmit_time);
        let request = NtpPacket::request().with_transmit_time(ntp_transmit_time);
        let request = request.pack().map_err(|err| {
            #[cfg(feature = "std")]
            error!("Failed to pack NTP request packet: {:?}", err);
            ClockError::Io
        })?;

        // trim request to NTP_MIN_PACKET_LEN
        if request.len() < NTP_MIN_PACKET_LEN {
            return Err(ClockError::Io);
        }
        let request = &request[..NTP_MIN_PACKET_LEN];
        socket
            .send_to(request, self.server)
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

#[cfg(feature = "std")]
fn unix_nanos_to_ntp_timestamp(unix_nanos: u64) -> u64 {
    let unix_seconds = (unix_nanos / 1_000_000_000) as i128;
    let nanos = (unix_nanos % 1_000_000_000) as i128;
    let ntp_seconds = unix_seconds + NTP_UNIX_EPOCH as i128;
    let fraction = (nanos << 32) / 1_000_000_000i128;
    ((ntp_seconds as u64) << 32) | (fraction as u64)
}

fn ntp_timestamp_to_unix_nanos(ntp_timestamp: u64) -> u64 {
    if ntp_timestamp == 0 {
        return 0;
    }
    let seconds = (ntp_timestamp >> 32) as i128;
    let fraction = (ntp_timestamp & 0xffff_ffff) as i128;
    let unix_seconds = seconds - NTP_UNIX_EPOCH as i128;
    if unix_seconds < 0 {
        return 0;
    }
    let nanos = (fraction * 1_000_000_000i128) >> 32;
    let unix_nanos = unix_seconds * 1_000_000_000i128 + nanos;
    unix_nanos as u64
}

/// Returns the NTP time and offset in nanoseconds.
pub fn parse_ntp_packet(packet: &[u8], _local_time: u64) -> Result<NtpPacket, ClockError> {
    if packet.len() < NTP_MIN_PACKET_LEN {
        return Err(ClockError::PacketTooShort);
    }
    #[cfg(all(any(debug_assertions, test), feature = "std"))]
    {
        log::debug!("NTP packet length: {}", packet.len());
        log::debug!(
            "NTP Packet: {:?}",
            packet
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
        );
    }
    let mut result = [0u8; 60];
    result[..packet.len()].copy_from_slice(packet);
    let mut res = NtpPacket::unpack_from_slice(&result).map_err(|_| ClockError::InvalidResponse)?;

    if res.version < 1 || res.version > 4 {
        return Err(ClockError::InvalidVersion);
    }

    res.ref_time = ntp_timestamp_to_unix_nanos(res.ref_time);
    res.origin_time = ntp_timestamp_to_unix_nanos(res.origin_time);
    res.recv_time = ntp_timestamp_to_unix_nanos(res.recv_time);
    res.transmit_time = ntp_timestamp_to_unix_nanos(res.transmit_time);

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

    use crate::constants::NTP_PORT;
    let addrs: Vec<SocketAddr> = (server, NTP_PORT).to_socket_addrs()?.collect();
    if let Some(addr) = addrs.first() {
        Ok(*addr)
    } else {
        Err(ClockError::ConfigError(format!(
            "Could not resolve NTP server address: {}",
            server
        )))
    }
}
