use ntp_clock::{error::ClockError, packets::NtpPacket, parse_ntp_packet};
use packed_struct::PackedStruct;

const UNIX_NANOS_SAMPLE: u64 = 1_735_689_600_000_000_000;

fn unix_nanos_to_ntp_timestamp(unix_nanos: u64) -> u64 {
    const NTP_UNIX_EPOCH: i128 = 2_208_988_800;
    let unix_seconds = (unix_nanos / 1_000_000_000) as i128;
    let nanos = (unix_nanos % 1_000_000_000) as i128;
    let ntp_seconds = unix_seconds + NTP_UNIX_EPOCH;
    let fraction = (nanos << 32) / 1_000_000_000i128;
    ((ntp_seconds as u64) << 32) | (fraction as u64)
}

#[test]
fn parse_packet_valid() {
    let local_time = UNIX_NANOS_SAMPLE;
    let ntp_timestamp = unix_nanos_to_ntp_timestamp(local_time);
    let mut packet = NtpPacket::request();
    packet.ref_time = ntp_timestamp;
    packet.origin_time = ntp_timestamp;
    packet.recv_time = ntp_timestamp;
    packet.transmit_time = ntp_timestamp;
    let packet = packet.pack().expect("Should pack NTP response");
    let response = parse_ntp_packet(&packet, local_time).expect("Failed to parse valid NTP packet");
    dbg!(&response);
    assert_eq!(response.offset_from_local(local_time), 0);
}

#[test]
fn parse_packet_invalid() {
    let local_time = UNIX_NANOS_SAMPLE;
    let packet = [0u8; 12];
    let parsed = parse_ntp_packet(&packet, local_time);
    dbg!(&parsed);
    assert!(matches!(parsed, Err(ClockError::PacketTooShort)));
    let packet = [0u8; 48];
    let parsed = parse_ntp_packet(&packet, local_time);
    dbg!(&parsed);
    assert!(matches!(parsed, Err(ClockError::InvalidVersion)));
}
