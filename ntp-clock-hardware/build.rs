use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    let memory_x = out_dir.join("memory.x");
    let mut contents = fs::read_to_string("memory.x").expect("failed to read memory.x");
    if let Some(origin) = flash_origin() {
        let replacement = format!("FLASH : ORIGIN = {origin}, LENGTH = 0x400000");
        contents = contents.replace(
            "FLASH : ORIGIN = 0x10000000, LENGTH = 0x400000",
            &replacement,
        );
    }
    let mut file = fs::File::create(&memory_x).expect("failed to create memory.x");
    file.write_all(contents.as_bytes())
        .expect("failed to write memory.x");
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-env-changed=PICO2W_FLASH_OFFSET");
    println!("cargo:rerun-if-env-changed=PICO2W_FLASH_ORIGIN");
}

fn flash_origin() -> Option<String> {
    if let Ok(origin) = env::var("PICO2W_FLASH_ORIGIN") {
        return Some(origin);
    }
    if let Ok(offset) = env::var("PICO2W_FLASH_OFFSET") {
        let offset = parse_hex(&offset)?;
        let origin = 0x1000_0000u32.checked_add(offset)?;
        return Some(format!("0x{origin:08x}"));
    }
    None
}

fn parse_hex(input: &str) -> Option<u32> {
    let trimmed = input.trim();
    let value = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    u32::from_str_radix(value, 16).ok()
}
