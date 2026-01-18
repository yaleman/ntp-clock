use ntp_clock::*;
use time::macros::datetime;
use time::{Duration as TimeDuration, OffsetDateTime};

#[test]
fn parse_packet_valid() {
    let local_time = datetime!(2025-01-01 00:00:00 +0);
    let packet = build_ntp_response(local_time);
    let parsed = NtpClient::parse_packet(&packet, local_time);
    assert!(parsed.is_ok());
    let (ntp_time, offset) = parsed.ok().unwrap_or((local_time, TimeDuration::ZERO));
    assert_eq!(ntp_time, local_time);
    assert_eq!(offset, TimeDuration::ZERO);
}

#[test]
fn parse_packet_invalid() {
    let local_time = datetime!(2025-01-01 00:00:00 +0);
    let packet = [0u8; 12];
    let parsed = NtpClient::parse_packet(&packet, local_time);
    assert!(matches!(parsed, Err(ClockError::InvalidResponse)));
}

fn build_ntp_response(time: OffsetDateTime) -> [u8; 48] {
    let mut packet = [0u8; 48];
    let ntp_seconds = (time.unix_timestamp() + NTP_UNIX_EPOCH) as u32;
    let nanos = time.nanosecond() as u128;
    let fraction = ((nanos << 32) / 1_000_000_000u128) as u32;
    packet[40..44].copy_from_slice(&ntp_seconds.to_be_bytes());
    packet[44..48].copy_from_slice(&fraction.to_be_bytes());
    packet
}
