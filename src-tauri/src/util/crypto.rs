use std::process::Command;

#[cfg(target_os = "macos")]
pub fn get_board_serial() -> anyhow::Result<String> {
    let output = Command::new("system_profiler")
        .arg("SPHardwareDataType")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;

    match stdout
        .lines()
        .find(|line| line.contains("Serial Number (system)"))
        .and_then(|line| line.split_whitespace().nth(3))
    {
        Some(serial) => Ok(serial.to_string()),
        None => Err(anyhow::anyhow!(
            "Failed to extract serial number from system_profiler output"
        )),
    }
}

#[cfg(target_os = "linux")]
pub fn get_board_serial() -> anyhow::Result<String> {
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

    Err(anyhow::anyhow!("Could not retrieve board serial number"))
}

#[cfg(target_os = "windows")]
pub fn get_board_serial() -> anyhow::Result<String> {
    use std::os::windows::process::CommandExt;
    let output = Command::new("wmic")
        .args(["baseboard", "get", "serialnumber"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()?;
    let output = String::from_utf8(output.stdout)?;
    let mut lines = output.lines();
    lines.next();
    let serial = lines.next().map(|x| x.trim().to_string());
    serial.ok_or_else(|| anyhow::anyhow!("serial number not found in output"))
}

pub fn get_pcl_hash(str: &str) -> u64 {
    let mut result = 5381 as u64;
    for c in str.chars() {
        result = (result << 5) ^ result ^ c as u64;
    }
    result ^ 0xA98F501BC684032F as u64
}

#[test]
fn serial_test() {
    let serial = get_board_serial().unwrap();
    let pcl_hash = get_pcl_hash(&serial);
    println!("serial: {}, pcl_hash: {:?}", serial, pcl_hash);
}
