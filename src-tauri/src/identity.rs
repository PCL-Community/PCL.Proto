//! 设备标识；只属于启动器自身，核心库不参与。

use std::process::Command;

#[cfg(target_os = "macos")]
fn get_board_serial() -> Result<String, String> {
    let output = Command::new("system_profiler")
        .arg("SPHardwareDataType")
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;

    match stdout
        .lines()
        .find(|line| line.contains("Serial Number (system)"))
        .and_then(|line| line.split_whitespace().nth(3))
    {
        Some(serial) => Ok(serial.to_string()),
        None => Err("Failed to extract serial number from system_profiler output".into()),
    }
}

#[cfg(target_os = "linux")]
fn get_board_serial() -> Result<String, String> {
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
        && output.status.success()
    {
        let serial = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !serial.is_empty() && serial != "None" {
            return Ok(serial);
        }
    }

    Err("Could not retrieve board serial number".into())
}

#[cfg(target_os = "windows")]
fn get_board_serial() -> Result<String, String> {
    use std::os::windows::process::CommandExt;
    let output = Command::new("wmic")
        .args(["baseboard", "get", "serialnumber"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| e.to_string())?;

    let output = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let mut lines = output.lines();
    lines.next();
    let serial = lines.next().map(|x| x.trim().to_string());
    serial.ok_or_else(|| "serial number not found in output".to_owned())
}

/// 设备标识，对照 `PCL.Core.Utils.Secret.Identify`。
pub struct Identify;

impl Identify {
    /// 参与哈希的原始硬件标识。
    pub fn raw_id() -> Result<String, String> {
        get_board_serial()
    }

    /// 持久化在 `pcl_identifier.txt` 里的启动器标识。
    pub fn launcher_id() -> Result<String, String> {
        Ok(format!(
            "{:x}",
            pcl_core::utils::hash::get_pcl_hash(&Self::raw_id()?)
        ))
    }
}
