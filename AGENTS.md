# Repository Guidelines

## Project Structure & Module Organization

- `src/main.rs` contains the CLI entry point and wires up the async runtime.
- `src/lib.rs` exposes the `NtpClient` and error types used by the binary.
- `src/cli.rs` defines CLI flags and environment variable bindings.
- `src/prelude.rs` re-exports commonly used types.
- `target/` holds build artifacts and should not be edited.

## Build, Test, and Development Commands

- Use the `justfile` to standardize developer commands; run `just --list` to see available tasks.
- `just check` runs clippy, tests, and formatting in one pass.
- `just clippy` runs the Rust linter across all targets.
- `just test` runs the test suite.
- `just fmt` formats Rust sources using rustfmt.
- `just coverage` generates a tarpaulin HTML report at `tarpaulin-report.html`.
- `just coveralls` uploads coverage to Coveralls (requires `COVERALLS_REPO_TOKEN`).
- `just semgrep` runs static analysis with Semgrep.

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
- Network access is required when real NTP querying is implemented; current logic is a placeholder.
- Always use the `cargo` commands for managing packages and enable networking when doing so.
