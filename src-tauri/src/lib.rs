mod commands;
mod notitg;
mod sometsuki;
mod state;

use std::sync::Mutex;
use tauri::{Manager};
use crate::state::AppStateInner;

pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      app.manage(Mutex::new(AppStateInner::default()));
      Ok(())
    })
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![commands::find_notitg_process, commands::connect, commands::send_message, commands::disconnect])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}