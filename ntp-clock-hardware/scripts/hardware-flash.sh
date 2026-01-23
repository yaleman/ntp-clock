#!/usr/bin/env bash
set -euo pipefail

: "${PICO2W_TARGET:=thumbv8m.main-none-eabihf}"
: "${PICO2W_PROFILE:=release}"
: "${PICO2W_MOUNT:=/Volumes/RPI-RP2}"

ELF_PATH="target/${PICO2W_TARGET}/${PICO2W_PROFILE}/ntp-clock-hardware"
UF2_PATH="${ELF_PATH}.uf2"

if [[ ! -f "${ELF_PATH}" ]]; then
  echo "Error: missing ELF at ${ELF_PATH}. Run hardware-build.sh first."
  exit 1
fi

if [[ ! -f "${UF2_PATH}" ]]; then
  if command -v elf2uf2-rs >/dev/null 2>&1; then
    if ! elf2uf2-rs "${ELF_PATH}" "${UF2_PATH}"; then
      echo "elf2uf2-rs failed; trying picotool."
    fi
  fi
fi

if [[ ! -f "${UF2_PATH}" ]] && command -v picotool >/dev/null 2>&1; then
  picotool uf2 convert --output "${UF2_PATH}" "${ELF_PATH}"
fi

if [[ ! -f "${UF2_PATH}" ]]; then
  echo "Error: unable to build UF2. Install elf2uf2-rs or picotool."
  exit 1
fi
cp "${UF2_PATH}" "${PICO2W_MOUNT}/"
