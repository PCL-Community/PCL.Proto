use std::process::Command;

pub mod file;
pub mod toys;

fn get_hash(str: &str) -> u64 {
    let mut result = 5381 as u64;
    for c in str.chars() {
        result = (result << 5) ^ result ^ c as u64;
    }
    result ^ 0xA98F501BC684032F as u64
}

#[cfg(target_os = "macos")]
fn get_board_serial() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("system_profiler")
        .arg("SPHardwareDataType")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    let serial = stdout
        .lines()
        .find(|line| line.contains("Serial Number (system)"))
        .unwrap()
        .split_whitespace()
        .nth(3)
        .unwrap()
        .to_string();
    Ok(serial)
}

#[cfg(target_os = "linux")]
fn get_board_serial() -> Result<String, Box<dyn std::error::Error>> {
    let serial = std::fs::read_to_string("/sys/class/dmi/id/product_serial")?;
    Ok(serial.trim().to_string())
}

#[test]
fn macos_platform_serial_test() {
    let serial = get_board_serial().unwrap();
    println!("{}", serial);
}
