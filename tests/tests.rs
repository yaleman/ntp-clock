use ntp_clock::{ClockError, NTP_UNIX_EPOCH, NtpClient};

const UNIX_NANOS_SAMPLE: u64 = 1_735_689_600_000_000_000;

#[test]
fn parse_packet_valid() {
    let local_time = UNIX_NANOS_SAMPLE;
    let packet = build_ntp_response(local_time);
    let parsed = NtpClient::parse_packet(&packet, local_time);
    assert!(parsed.is_ok());
    let (ntp_time, offset) = parsed.ok().unwrap_or((0, 0));
    assert_eq!(ntp_time, local_time);
    assert_eq!(offset, 0);
}

#[test]
fn parse_packet_invalid() {
    let local_time = UNIX_NANOS_SAMPLE;
    let packet = [0u8; 12];
    let parsed = NtpClient::parse_packet(&packet, local_time);
    assert!(matches!(parsed, Err(ClockError::InvalidResponse)));
}

fn build_ntp_response(unix_nanos: u64) -> [u8; 48] {
    let mut packet = [0u8; 48];
    let unix_seconds = unix_nanos / 1_000_000_000;
    let nanos = (unix_nanos % 1_000_000_000) as u128;
    let ntp_seconds = (unix_seconds as i64 + NTP_UNIX_EPOCH) as u32;
    let fraction = ((nanos << 32) / 1_000_000_000u128) as u32;
    packet[40..44].copy_from_slice(&ntp_seconds.to_be_bytes());
    packet[44..48].copy_from_slice(&fraction.to_be_bytes());
    packet
}
