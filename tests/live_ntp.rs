use ntp_clock::{NTP_UNIX_EPOCH, NtpClient};
use time::OffsetDateTime;

#[tokio::test]
async fn live_ntp_time_is_reasonable() {
    let sandbox_network_disabled =
        std::env::var("CODEX_SANDBOX_NETWORK_DISABLED").as_deref() == Ok("1");
    let ci = std::env::var("CI").as_deref() == Ok("1");
    if sandbox_network_disabled || ci {
        let client = NtpClient::new("127.0.0.1:123")
            .await
            .expect("client should parse server");
        let local_time = OffsetDateTime::now_utc();
        let response = build_ntp_response(local_time);
        let ntp_time = client
            .update_from_response(&response, local_time)
            .await
            .expect("mocked NTP update should succeed");

        let delta = (ntp_time - local_time).whole_seconds().abs();
        assert!(
            delta <= 600,
            "mocked ntp time drift too large: {delta} seconds"
        );

        let offset = client.get_offset().await;
        let offset_seconds = offset.whole_seconds().abs();
        assert!(
            offset_seconds <= 600,
            "mocked ntp offset too large: {offset_seconds} seconds"
        );
        return;
    }

    let client = NtpClient::new("au.pool.ntp.org:123")
        .await
        .expect("client should resolve server");
    let ntp_time = client.update().await.expect("NTP query should succeed");
    let local_time = OffsetDateTime::now_utc();
    let delta = (ntp_time - local_time).whole_seconds().abs();
    assert!(delta <= 60, "NTP time drift too large: {delta} seconds");

    let offset = client.get_offset().await;
    let offset_seconds = offset.whole_seconds().abs();
    assert!(
        offset_seconds <= 60,
        "stored NTP offset too large: {offset_seconds} seconds"
    );
    assert!(
        client.time_is_valid().await,
        "time should be valid after update"
    );
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
