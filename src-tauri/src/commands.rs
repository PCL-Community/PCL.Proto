use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn launch_game(app: AppHandle) {
  println!("[game] launch_game invoked from js.");
  app.emit("modal-open", "启动游戏功能尚未实现！").unwrap();
}