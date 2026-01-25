#![no_std]
#![no_main]

use core::str::FromStr;

use cyw43::{Control, JoinOptions};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use embassy_executor::Spawner;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Config, Ipv4Address, StackResources};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pwm::{Config as PwmConfig, Pwm, SetDutyCycle};
use embassy_time::{Duration, Timer};
use fixed::traits::ToFixed;
use ntp_clock::clock::hand_angles;
use ntp_clock::packets::NtpResponse;
use ntp_clock::parse_ntp_packet;
use ntp_clock_hardware::constants::NETWORK_DETAILS_LOG_DELAY_SECS;
use ntp_clock_hardware::hardware::{PwmServoController, ServoPwmConfig, angles_to_hand_degrees};
use ntp_clock_hardware::{ClockMechanism, LimitSwitches};
use panic_halt as _;
use static_cell::StaticCell;

const NTP_PORT: u16 = 123;
const NTP_PACKET_LEN: usize = 48;
const PWM_TARGET_HZ: u32 = 50;
const PWM_DIVIDER: u32 = 125;
const DEFAULT_NTP_SERVER: Ipv4Address = Ipv4Address::new(129, 6, 15, 28);
const DEFAULT_SYSLOG_PORT: u16 = 514;
const USB_ONLY: bool = option_env!("USB_ONLY").is_some();

const WIFI_SSID: &str = match option_env!("WIFI_SSID") {
    Some(value) => value,
    None => "",
};
const WIFI_PASSWORD: &str = match option_env!("WIFI_PASSWORD") {
    Some(value) => value,
    None => "",
};
const NTP_SERVER_ENV: &str = match option_env!("NTP_SERVER") {
    Some(value) => value,
    None => "",
};
const SYSLOG_SERVER_ENV: &str = match option_env!("SYSLOG_SERVER") {
    Some(value) => value,
    None => "",
};
const SYSLOG_PORT_ENV: &str = match option_env!("SYSLOG_PORT") {
    Some(value) => value,
    None => "",
};

bind_interrupts!(struct PioIrqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) {
    runner.run().await;
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) {
    runner.run().await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    if USB_ONLY {
        log::info!("USB-only firmware mode");
        usb_only_main(spawner, p).await;
    } else {
        full_main(spawner, p).await;
    }
}

async fn usb_only_main(spawner: Spawner, p: embassy_rp::Peripherals) -> ! {
    let usb = p.USB;
    ntp_clock_hardware::usb::init_usb_logging(&spawner, usb);
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

async fn full_main(spawner: Spawner, p: embassy_rp::Peripherals) {
    let usb = p.USB;
    ntp_clock_hardware::usb::init_usb_logging(&spawner, usb);

    let mut pio = Pio::new(p.PIO0, PioIrqs);
    let power = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let firmware = include_bytes!("../firmware/43439A0.bin");
    let clm = include_bytes!("../firmware/43439A0_clm.bin");

    let (net_device, mut control, runner) = cyw43::new(state, power, spi, firmware).await;
    let _ = spawner.spawn(wifi_task(runner));
    control.init(clm).await;

    if !WIFI_SSID.is_empty() && !WIFI_PASSWORD.is_empty() {
        connect_wifi(&mut control).await;
    } else {
        idle_missing_wifi().await;
    }

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let resources = RESOURCES.init(StackResources::new());
    let config = Config::dhcpv4(Default::default());
    let seed = 0x2f3a_9b5d;
    let (stack, runner) = embassy_net::new(net_device, config, resources, seed);
    let _ = spawner.spawn(net_task(runner));

    stack.wait_config_up().await;
    log::info!("DHCP configuration acquired");

    if let Some(syslog_server) = parse_ipv4(SYSLOG_SERVER_ENV) {
        let port = parse_u16(SYSLOG_PORT_ENV).unwrap_or(DEFAULT_SYSLOG_PORT);
        ntp_clock_hardware::usb::init_syslog_logging(&spawner, stack, syslog_server, port);
    } else if !SYSLOG_SERVER_ENV.is_empty() {
        log::warn!("SYSLOG_SERVER is not a valid IPv4 address");
    }

    let mut rx_meta = [PacketMetadata::EMPTY; 8];
    let mut rx_buffer = [0u8; 256];
    let mut tx_meta = [PacketMetadata::EMPTY; 8];
    let mut tx_buffer = [0u8; 256];
    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    if socket.bind(0).is_err() {
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }

    let server = ntp_server();

    let pwm_top = pwm_top_from_sysclk();
    let mut pwm_config = PwmConfig::default();
    pwm_config.divider = (PWM_DIVIDER as u16).to_fixed();
    pwm_config.top = pwm_top;

    let mut hour_pwm = Pwm::new_output_a(p.PWM_SLICE1, p.PIN_2, pwm_config.clone());
    let mut minute_pwm = Pwm::new_output_a(p.PWM_SLICE2, p.PIN_4, pwm_config);
    let servo_config = ServoPwmConfig::sg90_50hz();
    let hour_max = hour_pwm.max_duty_cycle() as u32;
    let minute_max = minute_pwm.max_duty_cycle() as u32;

    let controller = PwmServoController::new(
        |duty| hour_pwm.set_duty_cycle(duty.min(hour_max) as u16),
        |duty| minute_pwm.set_duty_cycle(duty.min(minute_max) as u16),
        servo_config,
        hour_max,
        minute_max,
    );
    let switches =
        LimitSwitchPins::new(Input::new(p.PIN_6, Pull::Up), Input::new(p.PIN_7, Pull::Up));
    let mut clock = ClockMechanism::new(controller, switches);

    let mut tick = 0u32;
    loop {
        if let Some(config) = stack.config_v4() {
            log::info!(
                "Net config: addr={}, gateway={:?}, dns={:?}",
                config.address,
                config.gateway,
                config.dns_servers
            );
        } else {
            log::info!("Net config: DHCP not ready");
        }

        if tick.is_multiple_of(4) {
            log::info!("Running NTP update against {}", server);
            if let Some(ntp_time) = query_ntp(&mut socket, server).await {
                log::info!("NTP update successful: {}", ntp_time.to_string());
                let angles = hand_angles(&ntp_time);
                let degrees = angles_to_hand_degrees(angles);
                let _ = clock.apply_hand_angles(degrees);
                clock.update_zeroing();
            } else {
                log::warn!("NTP update failed");
            }
        }
        tick = tick.wrapping_add(1);
        Timer::after(Duration::from_secs(NETWORK_DETAILS_LOG_DELAY_SECS)).await;
    }
}

async fn connect_wifi(control: &mut Control<'static>) {
    loop {
        let options = JoinOptions::new(WIFI_PASSWORD.as_bytes());
        log::info!("Joining WiFi SSID '{}'", WIFI_SSID);
        if control.join(WIFI_SSID, options).await.is_ok() {
            log::info!("WiFi joined");
            break;
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

async fn idle_missing_wifi() -> ! {
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}

async fn query_ntp(socket: &mut UdpSocket<'_>, server: Ipv4Address) -> Option<NtpResponse> {
    let mut request = [0u8; NTP_PACKET_LEN];
    request[0] = 0x1b;
    socket.send_to(&request, (server, NTP_PORT)).await.ok()?;
    let mut response = [0u8; NTP_PACKET_LEN];
    let (len, _) = socket.recv_from(&mut response).await.ok()?;
    parse_ntp_packet(&response[..len], 0).ok()
}

fn ntp_server() -> Ipv4Address {
    parse_ipv4(NTP_SERVER_ENV).unwrap_or(DEFAULT_NTP_SERVER)
}

fn parse_ipv4(input: &str) -> Option<Ipv4Address> {
    let mut octets = [0u8; 4];
    let mut parts = input.split('.');
    for slot in &mut octets {
        let part = parts.next()?;
        if part.is_empty() {
            return None;
        }
        *slot = u8::from_str(part).ok()?;
    }
    if parts.next().is_some() {
        return None;
    }
    Some(Ipv4Address::new(octets[0], octets[1], octets[2], octets[3]))
}

fn parse_u16(input: &str) -> Option<u16> {
    if input.is_empty() {
        return None;
    }
    input.parse::<u16>().ok()
}

fn pwm_top_from_sysclk() -> u16 {
    let sys_hz = clk_sys_freq();
    let denom = PWM_DIVIDER.saturating_mul(PWM_TARGET_HZ);
    let top = sys_hz.saturating_div(denom).saturating_sub(1);
    top.min(u16::MAX as u32) as u16
}

struct LimitSwitchPins<'d> {
    hour: Input<'d>,
    minute: Input<'d>,
}

impl<'d> LimitSwitchPins<'d> {
    fn new(hour: Input<'d>, minute: Input<'d>) -> Self {
        Self { hour, minute }
    }

    fn is_triggered(pin: &Input<'d>) -> bool {
        pin.is_low()
    }
}

impl<'d> LimitSwitches for LimitSwitchPins<'d> {
    fn hour_triggered(&self) -> bool {
        Self::is_triggered(&self.hour)
    }

    fn minute_triggered(&self) -> bool {
        Self::is_triggered(&self.minute)
    }
}
