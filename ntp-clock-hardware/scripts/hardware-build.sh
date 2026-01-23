#!/usr/bin/env bash
set -euo pipefail

: "${PICO2W_TARGET:=thumbv8m.main-none-eabihf}"
: "${PICO2W_PROFILE:=release}"

echo "Target:  ${PICO2W_TARGET}"
echo "Profile: ${PICO2W_PROFILE}"

if [ "$(rustup target list --installed | grep -c "^${PICO2W_TARGET}$")" -eq 0 ]; then
  echo "Adding Rust target: ${PICO2W_TARGET}"
  rustup target add "${PICO2W_TARGET}"
fi

cargo build \
  -p ntp-clock-hardware \
  --target "$PICO2W_TARGET" \
  --profile "$PICO2W_PROFILE" \
  --features hardware

ELF_PATH="target/${PICO2W_TARGET}/${PICO2W_PROFILE}/ntp-clock-hardware"
UF2_PATH="${ELF_PATH}.uf2"

if [[ ! -f "${ELF_PATH}" ]]; then
  echo "Error: missing ELF at ${ELF_PATH}."
  exit 1
fi

if command -v elf2uf2-rs >/dev/null 2>&1; then
  if elf2uf2-rs "${ELF_PATH}" "${UF2_PATH}"; then
    exit 0
  fi
  echo "elf2uf2-rs failed; trying picotool."
fi

if command -v picotool >/dev/null 2>&1; then
  picotool uf2 convert --output "${UF2_PATH}" "${ELF_PATH}"
  exit 0
fi

echo "Error: unable to build UF2. Install elf2uf2-rs or picotool."
exit 1
