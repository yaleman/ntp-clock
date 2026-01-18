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

use std::process::ExitCode;

use log::LevelFilter;
use ntp_clock::{cli::Cli, clock::hand_angles, prelude::*};

#[tokio::main]
async fn main() -> Result<(), ExitCode> {
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
        .inspect_err(|err| {
            error!("Failed to create NTP client: {err}");
        })?;
    let time = client
        .get_time()
        .await
        .inspect_err(|err| error!("Failed to run update: {err}"))?;
    let offset = client.get_offset().await;
    info!(
        "NTP time from {}: {} (Offset: {}ns)",
        cliopts.ntp_server,
        time,
        offset.whole_nanoseconds()
    );
    if cliopts.show_angles {
        let angles = hand_angles(time);
        info!(
            "Hand angles (deg): hour={}, minute={}, second={}",
            angles.hour.round() as i64,
            angles.minute.round() as i64,
            angles.second.round() as i64
        );
    }

    Ok(())
}
