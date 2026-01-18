use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(long, default_value_t = false)]
    pub debug: bool,
    #[clap(env = "NTP_SERVER")]
    pub ntp_server: String,

    #[clap(long, default_value_t = false)]
    pub show_angles: bool,
}
