use ntp_clock::{NtpClient, packets::NtpResponse, unix_nanos_now};
use packed_struct::PackedStruct;

#[test]
fn live_ntp_time_is_reasonable() {
    let sandbox_network_disabled =
        std::env::var("CODEX_SANDBOX_NETWORK_DISABLED").as_deref() == Ok("1");
    let ci = std::env::var("CI").as_deref() == Ok("1");
    if sandbox_network_disabled || ci {
        let mut client = NtpClient::new("127.0.0.1").expect("client should parse server");
        let local_time = unix_nanos_now();
        let response = NtpResponse::from_nanos(local_time)
            .pack()
            .expect("should pack NTP response");
        let ntp_time = client
            .update_from_response(&response, local_time)
            .expect("mocked NTP update should succeed");

        let delta = ((ntp_time as i128 - local_time as i128).abs() / 1_000_000_000) as i128;
        assert!(
            delta <= 600,
            "mocked ntp time drift too large: {delta} seconds"
        );

        let offset = client
            .last_response
            .expect("response should be available")
            .offset_from_local(local_time);
        let offset_seconds = (offset as i128).abs() / 1_000_000_000;
        assert!(
            offset_seconds <= 600,
            "mocked ntp offset too large: {offset_seconds} seconds"
        );
        return;
    }

    let mut client = NtpClient::new("au.pool.ntp.org").expect("client should resolve server");
    client.update().expect("NTP update should succeed");
    println!(
        "NtpResponse: {:?}",
        client
            .last_response
            .as_ref()
            .expect("response should be available")
    );
    let local_time = unix_nanos_now();
    let offset = client
        .last_response
        .as_ref()
        .expect("response should be available")
        .offset_from_local(local_time);
    assert!(offset <= 60, "NTP time drift too large: {offset} seconds");

    let last_response = client
        .last_response
        .clone()
        .expect("response should be available");

    let offset = last_response.offset_from_local(local_time);
    // let offset_seconds = (offset as i128).abs() / 1_000_000_000;

    assert!(
        offset <= 60,
        "stored NTP offset too large: {offset} seconds"
    );
    assert!(client.time_is_valid(), "time should be valid after update");
}
