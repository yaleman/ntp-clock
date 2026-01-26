#!/usr/bin/env bash
set -euo pipefail

: "${PICO2W_TARGET:=thumbv8m.main-none-eabihf}"
: "${PICO2W_PROFILE:=release}"
: "${PICO2W_UF2_FAMILY:=0xe48bff5b}"

echo "Target:  ${PICO2W_TARGET}"
echo "Profile: ${PICO2W_PROFILE}"

if [ "$(rustup target list --installed | grep -c "^${PICO2W_TARGET}$")" -eq 0 ]; then
  echo "Adding Rust target: ${PICO2W_TARGET}"
  rustup target add "${PICO2W_TARGET}"
fi

cargo build \
  -p ntp-clock-hardware \
  --target "$PICO2W_TARGET" \
  --profile "$PICO2W_PROFILE"

ELF_PATH="target/${PICO2W_TARGET}/${PICO2W_PROFILE}/ntp-clock-hardware"
UF2_PATH="${ELF_PATH}.uf2"

if [[ ! -f "${ELF_PATH}" ]]; then
  echo "Error: missing ELF at ${ELF_PATH}."
  exit 1
fi

if command -v picotool >/dev/null 2>&1; then
  if picotool uf2 convert --family "${PICO2W_UF2_FAMILY}" -t elf "${ELF_PATH}" "${UF2_PATH}"; then
    if [[ -z "${PICO2W_FLASH_OFFSET:-}" && -z "${PICO2W_FLASH_ORIGIN:-}" ]]; then
      picotool info -a "${UF2_PATH}"
    fi
    exit 0
  fi
  echo "picotool failed to build UF2."
  exit 1
fi

if command -v elf2uf2-rs >/dev/null 2>&1; then
  if elf2uf2-rs "${ELF_PATH}" "${UF2_PATH}"; then
    exit 0
  fi
  echo "elf2uf2-rs failed."
  exit 1
fi

echo "Error: unable to build UF2. Install elf2uf2-rs or picotool."
exit 1
