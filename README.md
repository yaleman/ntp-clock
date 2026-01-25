# ntp-clock

Rust workspace for an NTP-driven analog clock. It contains a host CLI for
querying NTP servers and a Raspberry Pi Pico 2 W firmware image that drives
servos and limit switches.

## Crates

- `ntp-clock`: std-enabled library + CLI (`cargo run -p ntp-clock`).
- `ntp-clock-hardware`: no-std firmware for Pico 2 W (`just build` / `just flash`).

## Host CLI

Provide the NTP server as a positional argument or via `NTP_SERVER`:

```bash
cargo run -p ntp-clock -- pool.ntp.org
NTP_SERVER=time.nist.gov cargo run -p ntp-clock -- --show-angles
```

Flags:

- `--debug` enables debug logging.
- `--show-angles` logs computed hand angles.

## Hardware Firmware

See `ntp-clock-hardware/README.md` for wiring, firmware builds, and flashing.
In short, firmware builds require a Pico 2 W target, CYW43439 firmware blobs,
and a UF2 tool (`picotool` or `elf2uf2-rs`).

## Development

The `justfile` defines common tasks:

- `just check` runs clippy, tests, formatting, shellcheck, firmware build, and
  semgrep; it also runs a `cargo check` for the hardware crate.
- `just build` builds the Pico 2 W firmware image.
- `just firmware` downloads the CYW43439 Wi-Fi firmware blobs.
- `just test` runs the host crate tests.

`just check` and `just build` require network access for firmware downloads.
