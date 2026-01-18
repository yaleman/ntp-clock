use ntp_clock::NtpClient;
use time::OffsetDateTime;

#[tokio::test]
async fn live_ntp_time_is_reasonable() {
    let client = NtpClient::new("au.pool.ntp.org:123")
        .await
        .expect("client should resolve server");
    let ntp_time = client.update().await.expect("ntp query should succeed");
    let local_time = OffsetDateTime::now_utc();
    let delta = (ntp_time - local_time).whole_seconds().abs();
    assert!(
        delta <= 600,
        "ntp time drift too large: {delta} seconds"
    );
}
