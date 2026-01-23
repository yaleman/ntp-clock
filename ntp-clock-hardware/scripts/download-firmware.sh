#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
firmware_dir="${root_dir}/firmware"

mkdir -p "${firmware_dir}"

base_url="https://raw.githubusercontent.com/embassy-rs/cyw43/master/firmware"

firmware_bin="${firmware_dir}/43439A0.bin"
firmware_clm="${firmware_dir}/43439A0_clm.bin"
expected_bin="4171a0906cad80c7ee6398a80233b08f0089b65c021ecb9ffed33f02b6ed3c5b"
expected_clm="27f9abd62bd92858d54e4825f791e27f65b6a4ce94eafc3c729d4760ce507762"

if [[ -s "${firmware_bin}" && -s "${firmware_clm}" && "${FORCE:-0}" != "1" ]]; then
    echo "Firmware already present in ${firmware_dir} (set FORCE=1 to re-download)."
    exit 0
fi

curl -fsSL "${base_url}/43439A0.bin" -o "${firmware_bin}"
curl -fsSL "${base_url}/43439A0_clm.bin" -o "${firmware_clm}"

hash_file() {
    shasum -a 256 "$1" | awk '{print $1}'
}

hash_bin="$(hash_file "${firmware_bin}")"
hash_clm="$(hash_file "${firmware_clm}")"

if [[ "${hash_bin}" != "${expected_bin}" || "${hash_clm}" != "${expected_clm}" ]]; then
    echo "Firmware hash mismatch. Delete ${firmware_dir} contents and re-download."
    exit 1
fi

echo "Downloaded CYW43439 firmware to ${firmware_dir} (hash verified)"
