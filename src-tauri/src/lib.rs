mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            #[cfg(target_os = "windows")]
            {
                // Windows 特供窗口设置
                let main_window = app.get_webview_window("main").unwrap();
                main_window.set_decorations(true)?;
                main_window.set_title("PCL.Proto")?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::launch_game])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
