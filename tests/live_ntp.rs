use ntp_clock::NtpClient;
use time::OffsetDateTime;

#[tokio::test]
async fn live_ntp_time_is_reasonable() {
    let client = NtpClient::new("au.pool.ntp.org:123")
        .await
        .expect("client should resolve server");
    let ntp_time = client.update().await.expect("NTP query should succeed");
    let local_time = OffsetDateTime::now_utc();
    let delta = (ntp_time - local_time).whole_seconds().abs();
    assert!(delta <= 600, "NTP time drift too large: {delta} seconds");

    let offset = client.get_offset().await;
    let offset_seconds = offset.whole_seconds().abs();
    assert!(
        offset_seconds <= 600,
        "stored NTP offset too large: {offset_seconds} seconds"
    );
}
