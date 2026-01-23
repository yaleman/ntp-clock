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
- CYW43439 firmware blobs placed at `ntp-clock-hardware/firmware/43439A0.bin`
  and `ntp-clock-hardware/firmware/43439A0_clm.bin`.

## Build

Use the helper script to build a firmware image for Pico 2 W:

```bash
just firmware
export WIFI_SSID=your-ssid
export WIFI_PASSWORD=your-password
export NTP_SERVER=129.6.15.28     # optional IPv4 literal
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
export NTP_SERVER=129.6.15.28     # optional IPv4 literal
PICO2W_MOUNT=/Volumes/RPI-RP2 ./scripts/hardware-flash.sh
```

Optional settings:

- `PICO2W_TARGET` (default `thumbv8m.main-none-eabihf`)
- `PICO2W_PROFILE` (default `release`)
- `PICO2W_MOUNT` (default `/Volumes/RPI-RP2`)

## Wiring Notes

- Default pin mapping (adjust in `ntp-clock-hardware/src/main.rs` if needed):
  - Hour servo PWM: GPIO2
  - Minute servo PWM: GPIO4
  - Hour limit switch: GPIO6 (active-low with pull-up)
  - Minute limit switch: GPIO7 (active-low with pull-up)
- Connect the hour and minute servos to independent PWM channels.
- Wire each limit switch to a GPIO with pull-ups/pull-downs as required.
- Call `ClockMechanism::update_zeroing()` when a switch triggers to zero that hand.
- Wi-Fi credentials are compiled in via `WIFI_SSID` and `WIFI_PASSWORD`.
- `NTP_SERVER` must be an IPv4 literal (DNS lookups are not configured).

## Firmware Blobs

The CYW43439 Wi-Fi chip needs firmware loaded at boot. Download the blobs from
the embassy-rs `cyw43` repo and place them in `ntp-clock-hardware/firmware/`:

```bash
just firmware
```

The script fetches:

- `43439A0.bin`
- `43439A0_clm.bin`

It verifies the SHA-256 hashes and will fail if they do not match.
