git := require("git")
cargo := require("cargo")
build_target := "thumbv8m.main-none-eabihf"

default:
    just --list

# run the linter, tests, and format the code
check: clippy test fmt
    cargo check -p ntp-clock-hardware --all-features --target {{build_target}}
    find . -name '*.sh' | xargs shellcheck

# run clippy
clippy:
    cargo clippy --quiet -p ntp-clock-hardware --target {{build_target}} --bins

# run rust tests
test:
    cargo test --quiet -p ntp-clock

# format the rust code
fmt:
    cargo fmt --all

build: firmware
    ./ntp-clock-hardware/scripts/hardware-build.sh

flash: build
    ./ntp-clock-hardware/scripts/flash-pico.sh
    ./ntp-clock-hardware/scripts/screen.sh


firmware:
    ./ntp-clock-hardware/scripts/download-firmware.sh

set positional-arguments

[private]
@coverage_inner *args='':
    cargo tarpaulin --workspace --exclude-files=src/main.rs $@

# run coverage checks
coverage:
    just coverage_inner --out=Html
    @echo "Coverage report should be at file://$(pwd)/tarpaulin-report.html"

coveralls:
    just coverage_inner --out=Html --coveralls $COVERALLS_REPO_TOKEN
    @echo "Coverage report should be at https://coveralls.io/github/yaleman/ntp-clock?branch=$(git branch --show-current)"

semgrep:
    semgrep ci --config auto \
    --exclude-rule "yaml.github-actions.security.third-party-action-not-pinned-to-commit-sha.third-party-action-not-pinned-to-commit-sha"
