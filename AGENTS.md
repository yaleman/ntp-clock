# Repository Guidelines

You're not done with a task until `just check` and `just build` finishes without errors or warnings.

Unless explicitly told to do so, stubbing out things to finish later is explicitly banned.

If the task is taking too long or you can't work it out, stop and ask for clarification or assistance.

Stop making things overly extensible - get it working to spec first and MAYBE offer to make it flexible if it's obvious this could be made better.

## Project Structure & Module Organization

- Workspace crates:
  - `ntp-clock/` holds the host CLI and shared NTP logic.
  - `ntp-clock-hardware/` holds the Pico 2 W firmware and hardware helpers.
- `ntp-clock/src/main.rs` contains the CLI entry point.
- `ntp-clock/src/lib.rs` exposes the `NtpClient` and error types used by the CLI.
- `ntp-clock/src/cli.rs` defines CLI flags and environment variable bindings.
- `ntp-clock/src/prelude.rs` re-exports commonly used types.
- `ntp-clock-hardware/src/main.rs` is the firmware entry point.
- `ntp-clock-hardware/src/lib.rs` exposes hardware helpers (servos, switches, logging).
- `ntp-clock-hardware/scripts/` contains build/flash helper scripts.
- `target/` holds build artifacts and should not be edited.

## Build, Test, and Development Commands

- Use the `justfile` to standardize developer commands; run `just --list` to see available tasks.
- `just check` runs clippy, tests, formatting, shellcheck, firmware build, and semgrep, then runs a `cargo check` for the hardware crate.
- `just clippy` runs the Rust linter across both crates.
- `just test` runs the host crate test suite.
- `just fmt` formats Rust sources using rustfmt.
- `just coverage` generates a tarpaulin HTML report at `tarpaulin-report.html`.
- `just coveralls` uploads coverage to Coveralls (requires `COVERALLS_REPO_TOKEN`).
- `just semgrep` runs static analysis with Semgrep.
- `just build` builds Pico 2 W firmware via `ntp-clock-hardware/scripts/hardware-build.sh`.
- `just flash` builds, flashes, and opens a serial console via `ntp-clock-hardware/scripts/flash-pico.sh` and `screen.sh`.
- `just firmware` downloads the CYW43439 firmware blobs for the hardware crate.

## Coding Style & Naming Conventions

- Rust edition is 2024; follow standard Rust style and formatting.
- Use 4-space indentation (rustfmt defaults) and avoid manual alignment.
- Prefer `snake_case` for functions/modules and `CamelCase` for types.
- Keep modules small and focused; place shared exports in `src/prelude.rs`.

## Testing Guidelines

- No test framework is configured beyond Rust’s built-in test harness.
- If adding tests, keep them in `src/` module tests or `tests/` integration tests.
- Name tests descriptively (for example `test_time_is_valid_after_update`).
- Use `just coverage` when adding tests to keep an eye on coverage gaps.

## Commit & Pull Request Guidelines

- There is only one commit (“initial commit”), so no established message convention.
- Use concise, imperative commit messages (for example “Add CLI flag parsing”).
- Pull requests should include:
  - A short summary of changes and rationale.
  - How to run or verify the change (commands or manual steps).
  - Notes about user-visible behavior changes.

## Configuration & Usage Notes

- The NTP server can be provided as a positional CLI argument or via `NTP_SERVER`.
- The host CLI can resolve hostnames; the hardware firmware only accepts IPv4 literals for `NTP_SERVER`.
- `WIFI_SSID` and `WIFI_PASSWORD` are compiled into the firmware; without them the firmware idles after USB logging starts.
- Syslog forwarding is controlled by `SYSLOG_SERVER` (IPv4 literal) and optional `SYSLOG_PORT`.
- Network access is required for NTP updates and for `just firmware` downloads.
- Always use the `cargo` commands for managing packages and enable networking when doing so.
- YOU ARE EXPLICITLY BANNED FROM MANUALLY EDITING Cargo.toml to change package definitions. USE CARGO THAT IS WHAT IT IS FOR.
