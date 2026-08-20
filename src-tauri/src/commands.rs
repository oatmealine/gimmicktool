use crate::sometsuki::{ConnectionEvent, Sometsuki};
use crate::notitg::{NOTITG_VERSIONS, NotITGError, VersionInfo, find_notitg_pid};

use process_memory::{Pid};
use serde_json::Value;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use crate::state::{AppState};

#[tauri::command]
pub fn find_notitg_process() -> Result<(Pid, &'static str, VersionInfo), NotITGError> {
  let (pid, ver) = find_notitg_pid()?;
  Ok((pid, ver, NOTITG_VERSIONS[ver]))
}

#[tauri::command]
pub fn connect(app_handle: AppHandle, pid: Pid, base_address: usize, size: usize, channel: Channel<ConnectionEvent>) -> Result<(), NotITGError> {
  let state = app_handle.state::<AppState>();
  let mut state = state.lock().unwrap();

  if state.sometsuki_tx.is_some() {
    // terminate old instance
    let _ = state.sometsuki_tx.as_mut().unwrap().send(());
  }

  state.sometsuki = None;
  
  let sometsuki = Sometsuki::new(pid, base_address, size, channel.clone())?;

  let (tx, rx) = mpsc::channel::<()>();

  state.sometsuki = Some(sometsuki);
  state.sometsuki_tx = Some(tx);

  let app_handle = app_handle.clone();

  thread::spawn(move || loop {
    let state = app_handle.state::<AppState>();
    let mut state = state.lock().unwrap();
    let Some(sometsuki) = state.sometsuki.as_mut() else {
      println!("ooh my sometsuki is gone. that's funny. explodes");
      break;
    };

    if sometsuki.is_closed() {
      sometsuki.log("closed, stopping thread");
      break;
    }

    match sometsuki.process() {
      Ok(_) => (),
      Err(e) => {
        sometsuki.log(&format!("err: {e}; dropping connection"));
        state.sometsuki = None;
        state.sometsuki_tx = None;
        channel.send(ConnectionEvent::Error { message: e.to_string() }).unwrap();
        channel.send(ConnectionEvent::Disconnected).unwrap();
        break;
      }
    }

    thread::park_timeout(Duration::from_millis(20));
    
    match rx.try_recv() {
      Ok(_) | Err(TryRecvError::Disconnected) => {
        sometsuki.log("rx received stop signal, stopping thread");
        break;
      }
      Err(TryRecvError::Empty) => {}
    }
  });

  Ok(())
}

#[tauri::command]
pub fn send_message(state: State<'_, AppState>, value: Value) -> Result<(), NotITGError> {
  let mut state = state.lock().unwrap();

  match state.sometsuki.as_mut() {
    None => Err(NotITGError::NotConnectedError),
    Some(sometsuki) => {
      sometsuki.send_message(&value)?;
      Ok(())
    }
  }
}

#[tauri::command]
pub fn disconnect(app_handle: AppHandle) {
  let state = app_handle.state::<AppState>();
  let mut state = state.lock().unwrap();

  if state.sometsuki_tx.is_some() {
    let _ = state.sometsuki_tx.as_mut().unwrap().send(());
  }
  
  if state.sometsuki.is_some() {
    state.sometsuki.as_mut().unwrap().disconnect(false);
  }
}