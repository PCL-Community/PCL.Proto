use std::process::Command;

pub mod file;
pub mod toys;

pub fn get_hash(str: &str) -> u64 {
    let mut result = 5381 as u64;
    for c in str.chars() {
        result = (result << 5) ^ result ^ c as u64;
    }
    result ^ 0xA98F501BC684032F as u64
}

#[cfg(target_os = "macos")]
pub fn get_board_serial() -> Result<String, Box<dyn std::error::Error>> {
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
    // 首先尝试 DMI 路径
    let dmi_paths = [
        "/sys/class/dmi/id/board_serial",
        "/sys/devices/virtual/dmi/id/board_serial",
    ];

    for path in &dmi_paths {
        if let Ok(serial) = std::fs::read_to_string(path) {
            let trimmed = serial.trim();
            if !trimmed.is_empty() && trimmed != "None" && trimmed != "To be filled by O.E.M." {
                return Ok(trimmed.to_string());
            }
        }
    }

    // 尝试设备树（适用于嵌入式设备）
    if let Ok(serial) = std::fs::read_to_string("/proc/device-tree/serial-number") {
        return Ok(serial.trim().to_string());
    }

    // 最后尝试 dmidecode（可能需要 sudo）
    if let Ok(output) = Command::new("dmidecode")
        .args(["-s", "baseboard-serial-number"])
        .output()
    {
        if output.status.success() {
            let serial = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !serial.is_empty() && serial != "None" {
                return Ok(serial);
            }
        }
    }

    Err("Could not retrieve board serial number".into())
}

#[test]
fn macos_platform_serial_test() {
    let serial = get_board_serial().unwrap();
    println!("{}", serial);
}
