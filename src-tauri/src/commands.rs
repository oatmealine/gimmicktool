use crate::sometsuki::{ConnectionEvent, SometsukiCommand};
use crate::notitg::{NOTITG_VERSIONS, NotITGError, VersionInfo, find_notitg_pid};

use log::error;
use process_memory::{Pid};
use serde_json::Value;
use tauri::ipc::Channel;
use tauri::State;
use crate::state::AppState;
use crate::message::Message;

fn report_if_errored<T>(res: Result<T, NotITGError>) -> Result<T, NotITGError> {
  if let Err(ref e) = res {
    error!("{e}");
  };
  res
}

#[tauri::command]
pub fn find_notitg_process() -> Result<(Pid, &'static str, &'static VersionInfo), NotITGError> {
  let (pid, ver) = report_if_errored(find_notitg_pid())?;
  Ok((pid, ver, &NOTITG_VERSIONS[ver]))
}

#[tauri::command]
pub fn connect(state: State<'_, AppState>, pid: Pid, base_address: usize, size: usize, channel: Channel<ConnectionEvent>) -> Result<(), NotITGError> {
  let (reply_tx, reply_rx) = oneshot::channel();
  state.sometsuki_tx.send(SometsukiCommand::Connect {
    reply: reply_tx, pid, base_address, size, channel
  })?;
  report_if_errored(reply_rx.recv()?)
}

#[tauri::command]
pub fn send_message(state: State<'_, AppState>, value: Value) -> Result<(), NotITGError> {
  let msg = Message::from_json(&value)?;
  let (reply_tx, reply_rx) = oneshot::channel();
  state.sometsuki_tx.send(SometsukiCommand::SendMessage {
    reply: reply_tx, value: msg
  })?;
  report_if_errored(reply_rx.recv()?)
}

#[tauri::command]
pub fn disconnect(state: State<'_, AppState>) -> Result<(), NotITGError> {
  state.sometsuki_tx.send(SometsukiCommand::Disconnect)?;
  Ok(())
}
