use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_net::Ipv4Address;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcAcmState};
use embassy_usb::{Builder, Config as UsbConfig, UsbDevice};
use heapless::String;
use static_cell::StaticCell;

bind_interrupts!(struct UsbIrqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type LogMessage = String<256>;

const USB_VID: u16 = 0xcafe;
const USB_PID: u16 = 0x4001;
static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
static USB_STATE: StaticCell<CdcAcmState> = StaticCell::new();
static USB_LOGGER: UsbLogger = UsbLogger;
static LOG_CHANNEL: Channel<CriticalSectionRawMutex, LogMessage, 8> = Channel::new();
static SYSLOG_CHANNEL: Channel<CriticalSectionRawMutex, LogMessage, 8> = Channel::new();

static SYSLOG_RX_META: StaticCell<[PacketMetadata; 1]> = StaticCell::new();
static SYSLOG_RX_BUFFER: StaticCell<[u8; 64]> = StaticCell::new();
static SYSLOG_TX_META: StaticCell<[PacketMetadata; 1]> = StaticCell::new();
static SYSLOG_TX_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();

#[embassy_executor::task]
async fn usb_logger_task(mut cdc: CdcAcmClass<'static, Driver<'static, USB>>) {
    loop {
        cdc.wait_connection().await;
        let message = LOG_CHANNEL.receive().await;
        for chunk in message.as_bytes().chunks(64) {
            let _ = cdc.write_packet(chunk).await;
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) {
    usb.run().await;
}

#[embassy_executor::task]
async fn syslog_task(socket: UdpSocket<'static>, server: Ipv4Address, port: u16) {
    loop {
        let message = SYSLOG_CHANNEL.receive().await;
        let _ = socket.send_to(message.as_bytes(), (server, port)).await;
    }
}

pub fn init_usb_logging(spawner: &Spawner, usb: embassy_rp::Peri<'static, USB>) {
    let driver = Driver::new(usb, UsbIrqs);
    let mut config = UsbConfig::new(USB_VID, USB_PID);
    config.manufacturer = Some("James Hodgkinson");
    config.product = Some("NTP Clock");
    config.serial_number = Some("ntp-clock");
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    let config_descriptor = CONFIG_DESCRIPTOR.init([0u8; 256]);
    let bos_descriptor = BOS_DESCRIPTOR.init([0u8; 256]);
    let msos_descriptor = MSOS_DESCRIPTOR.init([0u8; 256]);
    let control_buf = CONTROL_BUF.init([0u8; 128]);
    let mut builder = Builder::new(
        driver,
        config,
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
    );

    let cdc_state = USB_STATE.init(CdcAcmState::new());
    let cdc = CdcAcmClass::new(&mut builder, cdc_state, 64);

    let usb = builder.build();
    let _ = spawner.spawn(usb_task(usb));
    let _ = spawner.spawn(usb_logger_task(cdc));

    let _ = log::set_logger(&USB_LOGGER);
    log::set_max_level(log::LevelFilter::Info);
    log::info!("USB CDC-ACM logging enabled");
}

pub fn init_syslog_logging(
    spawner: &Spawner,
    stack: embassy_net::Stack<'static>,
    server: Ipv4Address,
    port: u16,
) {
    let rx_meta = SYSLOG_RX_META.init([PacketMetadata::EMPTY; 1]);
    let rx_buffer = SYSLOG_RX_BUFFER.init([0u8; 64]);
    let tx_meta = SYSLOG_TX_META.init([PacketMetadata::EMPTY; 1]);
    let tx_buffer = SYSLOG_TX_BUFFER.init([0u8; 256]);
    let mut socket = UdpSocket::new(stack, rx_meta, rx_buffer, tx_meta, tx_buffer);
    if socket.bind(0).is_err() {
        log::warn!("Failed to bind syslog UDP socket");
        return;
    }
    let _ = spawner.spawn(syslog_task(socket, server, port));
    log::info!("Syslog logging enabled: {}:{}", server, port);
}

struct UsbLogger;

impl log::Log for UsbLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut line = LogMessage::new();
        let _ = write!(&mut line, "[{}] {}\r\n", record.level(), record.args());
        let _ = LOG_CHANNEL.try_send(line);

        let priority = syslog_priority(record.level());
        let mut syslog_line = LogMessage::new();
        let _ = write!(
            &mut syslog_line,
            "<{}>ntp-clock-hardware: [{}] {}",
            priority,
            record.level(),
            record.args()
        );
        let _ = SYSLOG_CHANNEL.try_send(syslog_line);
    }

    fn flush(&self) {}
}

fn syslog_priority(level: log::Level) -> u8 {
    let facility = 1u8;
    let severity = match level {
        log::Level::Error => 3,
        log::Level::Warn => 4,
        log::Level::Info => 6,
        log::Level::Debug => 7,
        log::Level::Trace => 7,
    };
    facility * 8 + severity
}
