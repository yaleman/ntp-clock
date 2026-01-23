# ntp-clock-hardware

This crate provides hardware-focused helpers for driving a two-hand analog clock
using two servos and two limit switches. It pairs the generic hand-angle math
from the main `ntp-clock` crate with RP2350 PWM control and targets the Raspberry
Pi Pico 2 W (4MB flash). Wi-Fi uses the CYW43439 via the `cyw43` crate and the
`embassy-rp` stack, so the ARM target is required.

## Prerequisites

- Rust toolchain with the `thumbv8m.main-none-eabihf` target.
- `elf2uf2-rs` or `picotool` for producing UF2 images.
- A Raspberry Pi Pico 2 W wired to two servos and two limit switches.

## Build

Use the helper script to build a firmware image for Pico 2 W:

```bash
export WIFI_SSID=your-ssid
export WIFI_PASSWORD=your-password
export SYSLOG_HOST=192.168.1.10   # optional
export NTP_SERVER=129.6.15.28     # optional
./scripts/hardware-build.sh
```

This builds `ntp-clock-hardware` for the `thumbv8m.main-none-eabihf` target and
generates a UF2 alongside the ELF. Override defaults with `PICO2W_TARGET` or
`PICO2W_PROFILE` if needed. The script enables the `hardware` feature that pulls
in RP2350/Wi-Fi support.

## Flash

Flash by copying a UF2 to the mounted Pico 2 W boot volume:

```bash
export WIFI_SSID=your-ssid
export WIFI_PASSWORD=your-password
export SYSLOG_HOST=192.168.1.10   # optional
export NTP_SERVER=129.6.15.28     # optional
PICO2W_MOUNT=/Volumes/RPI-RP2 ./scripts/hardware-flash.sh
```

Optional settings:

- `PICO2W_TARGET` (default `thumbv8m.main-none-eabihf`)
- `PICO2W_PROFILE` (default `release`)
- `PICO2W_MOUNT` (default `/Volumes/RPI-RP2`)

## Wiring Notes

- Connect the hour and minute servos to independent PWM channels.
- Wire each limit switch to a GPIO with pull-ups/pull-downs as required.
- Call `ClockMechanism::update_zeroing()` when a switch triggers to zero that hand.
- Wi-Fi credentials are compiled in via `WIFI_SSID` and `WIFI_PASSWORD`. If
  `SYSLOG_HOST` is set to an IPv4 address, UDP syslog is sent on port 514.
