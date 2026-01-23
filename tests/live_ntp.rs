use ntp_clock::{NTP_UNIX_EPOCH, NtpClient, unix_nanos_now};

#[test]
fn live_ntp_time_is_reasonable() {
    let sandbox_network_disabled =
        std::env::var("CODEX_SANDBOX_NETWORK_DISABLED").as_deref() == Ok("1");
    let ci = std::env::var("CI").as_deref() == Ok("1");
    if sandbox_network_disabled || ci {
        let mut client = NtpClient::new("127.0.0.1").expect("client should parse server");
        let local_time = unix_nanos_now();
        let response = build_ntp_response(local_time);
        let ntp_time = client
            .update_from_response(&response, local_time)
            .expect("mocked NTP update should succeed");

        let delta = ((ntp_time as i128 - local_time as i128).abs() / 1_000_000_000) as i128;
        assert!(
            delta <= 600,
            "mocked ntp time drift too large: {delta} seconds"
        );

        let offset = client.get_offset();
        let offset_seconds = (offset as i128).abs() / 1_000_000_000;
        assert!(
            offset_seconds <= 600,
            "mocked ntp offset too large: {offset_seconds} seconds"
        );
        return;
    }

    let mut client = NtpClient::new("au.pool.ntp.org").expect("client should resolve server");
    let ntp_time = client.update().expect("NTP query should succeed");
    let local_time = unix_nanos_now();
    let delta = ((ntp_time as i128 - local_time as i128).abs() / 1_000_000_000) as i128;
    assert!(delta <= 60, "NTP time drift too large: {delta} seconds");

    let offset = client.get_offset();
    let offset_seconds = (offset as i128).abs() / 1_000_000_000;
    assert!(
        offset_seconds <= 60,
        "stored NTP offset too large: {offset_seconds} seconds"
    );
    assert!(client.time_is_valid(), "time should be valid after update");
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
