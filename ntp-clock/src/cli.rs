#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
use clap::Parser;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(long, default_value_t = false)]
    pub debug: bool,
    #[clap(env = "NTP_SERVER")]
    pub ntp_server: String,

    #[clap(long, default_value_t = false)]
    pub show_angles: bool,
}
