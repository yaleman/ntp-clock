#[cfg(any(target_family = "unix", target_family = "windows"))]
use clap::Parser;

#[cfg(any(target_family = "unix", target_family = "windows"))]
#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(long, default_value_t = false)]
    pub debug: bool,
    #[clap(env = "NTP_SERVER")]
    pub ntp_server: String,

    #[clap(long, default_value_t = false)]
    pub show_angles: bool,
}
