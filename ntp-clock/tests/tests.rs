use ntp_clock::{error::ClockError, packets::NtpPacket, parse_ntp_packet};
use packed_struct::PackedStruct;

const UNIX_NANOS_SAMPLE: u64 = 1_735_689_600_000_000_000;

#[test]
fn parse_packet_valid() {
    let local_time = UNIX_NANOS_SAMPLE;
    let packet = NtpPacket::from_nanos(local_time)
        .pack()
        .expect("Should pack NTP response");
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
