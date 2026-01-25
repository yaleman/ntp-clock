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

#[cfg(any(target_family = "unix", target_family = "windows"))]
fn cli_main() -> Result<(), ExitCode> {
    use clap::Parser;
    use ntp_clock::packets::NtpPacket;
    use ntp_clock::{cli::Cli, clock::hand_angles, prelude::*};

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

    let mut client = NtpClient::new(&cliopts.ntp_server).inspect_err(|err| {
        error!("Failed to create NTP client: {err}");
    })?;
    let time = client
        .get_time()
        .inspect_err(|err| error!("Failed to run update: {err}"))?;
    let offset = client
        .last_response
        .as_ref()
        .map(|resp| resp.offset_from_local(unix_nanos_now()))
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unknown".into());
    let seconds = time / 1_000_000_000;
    let nanos = time % 1_000_000_000;
    info!(
        "NTP time from {}: {}.{:09} UTC (Offset: {}ns)",
        cliopts.ntp_server, seconds, nanos, offset
    );
    if cliopts.show_angles {
        let angles = hand_angles(&NtpPacket::from_nanos(time));
        info!(
            "Hand angles (deg): hour={}, minute={}, second={}",
            angles.hour.round() as i64,
            angles.minute.round() as i64,
            angles.second.round() as i64
        );
    }
    Ok(())
}

fn main() -> Result<(), ExitCode> {
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    cli_main()?;
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    eprintln!("This binary was built without CLI support.");
    Ok(())
}
