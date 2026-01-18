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

use log::LevelFilter;
use ntp_clock::{cli::Cli, prelude::*};

#[tokio::main]
async fn main() -> Result<(), ClockError> {
    let cliopts = Cli::parse();

    let level = if cliopts.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    #[allow(clippy::expect_used)]
    simple_logger::SimpleLogger::new()
        .with_level(level)
        .without_timestamps()
        .init()
        .expect("Failed to initialize logger");

    let client = NtpClient::new(&cliopts.ntp_server)
        .await
        .inspect_err(|err| log::error!("Failed to create NTP client: {err:?}"))?;
    let time = client.get_time().await?;
    let offset = client.get_offset().await;
    log::info!(
        "NTP time from {}: {} (Offset: {}ns)",
        cliopts.ntp_server,
        time,
        offset.whole_nanoseconds()
    );

    Ok(())
}
