use terracotta::{controller, rooms::Room};

#[tauri::command]
pub fn get_terracotta_meta() -> serde_json::Value {
    todo!()
}

#[tauri::command]
pub fn get_terracotta_state() -> serde_json::Value {
    controller::get_state()
}

#[tauri::command]
pub fn set_terracotta_waiting() {
    controller::set_waiting()
}

#[tauri::command]
pub fn set_terracotta_host_scanning(player: String) {
    controller::set_scanning(None, Some(player));
}

#[tauri::command]
pub fn set_terracotta_guesting(room_code: String, player: String) -> Result<(), String> {
    let room = Room::from(&room_code).ok_or("invalid room code")?;
    if controller::set_guesting(room, Some(player)) {
        Ok(())
    } else {
        Err("set guesting failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use terracotta::rooms::Room;

    #[test]
    fn test_parse_code() {
        let codes = [
            "U/LX2M-2A87-YXMZ-2HJJ",
            "U/YS3D-LTH4-6AUC-MBFB",
            "U/BC4D-A51Z-ZE3P-LAP9",
            "U/UCAW-UY61-QD93-UL9X",
        ];
        for code in codes {
            let room = Room::from(code);
            assert!(room.is_some());
        }
    }
}
