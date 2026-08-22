mod commands;
mod notitg;
mod sometsuki;
mod state;

use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;

use tauri::{Manager};
use crate::sometsuki::{Sometsuki, SometsukiCommand};
use crate::state::AppState;

// 100hz seems reasonable
const POLL_RATE: Duration = Duration::from_millis(1_000 / 100);

pub fn run() {
  pretty_env_logger::init();

  tauri::Builder::default()
    .setup(|app| {
      let (tx, rx) = std::sync::mpsc::channel::<SometsukiCommand>();

      thread::spawn(move || {
        let mut sometsuki = Sometsuki::new();

        loop {
          match rx.try_recv() {
            Ok(cmd) => match cmd {
              SometsukiCommand::Connect {
                reply,
                pid, base_address, size, channel
              } => {
                reply.send(
                  sometsuki.connect(pid, base_address, size, channel)
                ).unwrap();
              },
              SometsukiCommand::SendMessage {
                reply, value
              } => {
                reply.send(
                  sometsuki.send_message(&value)
                ).unwrap();
              },
              SometsukiCommand::Disconnect => {
                sometsuki.disconnect(false);
              }
            },
            Err(TryRecvError::Empty) => thread::park_timeout(POLL_RATE),
            Err(TryRecvError::Disconnected) => panic!("mpsc channel got disconnected, somehow?"),
          }
          
          if !sometsuki.is_closed() {
            sometsuki.try_process();
          }
        }
      });

      app.manage(AppState { sometsuki_tx: tx });

      Ok(())
    })
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![commands::find_notitg_process, commands::connect, commands::send_message, commands::disconnect])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}