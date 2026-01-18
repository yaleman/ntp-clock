use ntp_clock::{cli::Cli, prelude::*};

#[tokio::main]
async fn main() -> Result<(), ClockError> {
    let cliopts = Cli::parse();

    if cliopts.debug {
        println!("Debug mode is ON");
    }

    let client = NtpClient::new(&cliopts.ntp_server)
        .await
        .expect("Failed to create NTP client");
    let time = client.get_time().await?;
    println!("NTP time from {}: {}", cliopts.ntp_server, time);

    Ok(())
}
