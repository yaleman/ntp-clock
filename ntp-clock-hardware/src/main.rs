#![no_std]
#![no_main]

use panic_halt as _;

#[cfg(target_arch = "arm")]
use cortex_m_rt::entry;
#[cfg(target_arch = "riscv32")]
use riscv_rt::entry;
use ntp_clock_hardware::hardware::{PwmServoController, ServoPwmConfig};
use ntp_clock_hardware::{ClockMechanism, HandAnglesDeg, LimitSwitches};

use cyw43 as _;
use cyw43_pio as _;
use rp235x_hal as _;

#[entry]
fn main() -> ! {
    // TODO: Initialize RP2350 clocks, GPIOs, PWM, and CYW43439 Wi-Fi via cyw43.
    let servo_config = ServoPwmConfig::sg90_50hz();
    let controller = PwmServoController::new(
        |_duty| Ok::<_, core::convert::Infallible>(()),
        |_duty| Ok::<_, core::convert::Infallible>(()),
        servo_config,
        65_535,
        65_535,
    );

    let switches = DummyLimitSwitches::default();
    let mut clock = ClockMechanism::new(controller, switches);

    loop {
        let angles = HandAnglesDeg {
            hour: 90.0,
            minute: 180.0,
        };
        let _ = clock.apply_hand_angles(angles);
        clock.update_zeroing();
    }
}

#[derive(Default)]
struct DummyLimitSwitches;

impl LimitSwitches for DummyLimitSwitches {
    fn hour_triggered(&self) -> bool {
        false
    }

    fn minute_triggered(&self) -> bool {
        false
    }
}
