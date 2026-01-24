#!/usr/bin/env bash
set -euo pipefail

MOUNT_POINT="${PICO_MOUNT_POINT:-/Volumes/RP2350}"
TARGET="${PICO2W_TARGET:-thumbv8m.main-none-eabihf}"
PROFILE="${PICO2W_PROFILE:-release}"
FIRMWARE_PATH="${FIRMWARE_PATH:-target/${TARGET}/${PROFILE}/ntp-clock-hardware.uf2}"
ELF_PATH="${ELF_PATH:-target/${TARGET}/${PROFILE}/ntp-clock-hardware}"

if [[ ! -d "${MOUNT_POINT}" ]]; then
  echo "Raspberry Pi mount not found at ${MOUNT_POINT}."
  exit 2
fi

if [[ ! -f "${FIRMWARE_PATH}" ]]; then
  echo "Firmware not found at ${FIRMWARE_PATH}."
  echo "Build it with: just firmware"
  exit 1
fi

echo "Flashing firmware to ${MOUNT_POINT}..."
if command -v picotool >/dev/null 2>&1; then
  if [[ ! -f "${ELF_PATH}" ]]; then
    echo "ELF not found at ${ELF_PATH} (needed for picotool)."
    exit 1
  fi
  if picotool load --family 0xe48bff57 -t elf "${ELF_PATH}"; then
    picotool reboot
    echo "Loaded ${ELF_PATH} via picotool."
    exit 0
  fi
  echo "picotool load failed; falling back to UF2 copy."
fi

dest_path="${MOUNT_POINT}/$(basename "${FIRMWARE_PATH}")"
if [[ -d "${dest_path}" ]]; then
  dest_path="${MOUNT_POINT}/firmware.uf2"
fi
cp -Xf "${FIRMWARE_PATH}" "${dest_path}"
echo "Copied ${FIRMWARE_PATH} to ${MOUNT_POINT}/"
