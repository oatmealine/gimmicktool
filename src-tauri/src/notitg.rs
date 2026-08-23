use log::info;
use process_memory::{DataMember, Memory, Pid, ProcessHandle, copy_address, TryIntoProcessHandle};
use sysinfo::System;
use std::io::ErrorKind;
use std::{collections::HashMap};
use std::sync::LazyLock;
use serde::ser::{Serialize, Serializer};
use regex::regex;

use crate::sometsuki::SometsukiCommand;

#[derive(Debug, thiserror::Error)]
pub enum NotITGError {
  #[error("I/O error: {0}")]
  Io(#[from] std::io::Error),

  #[error("JSON de/serialization error: {0}")]
  Json(#[from] serde_json::Error),

  #[error("UTF8 decoding error: {0}")]
  Utf8(#[from] std::str::Utf8Error),

  #[error("could not communicate with sometsuki thread: {0}")]
  ThreadReceiveError(#[from] oneshot::RecvError),
  #[error("could not communicate with sometsuki thread: {0}")]
  // TODO i'm not certain why this requires specifying the send type here
  ThreadSendError(#[from] std::sync::mpsc::SendError<SometsukiCommand>),
  
  // this is kind of a messy way to handle platform-specific error messages
  #[cfg(target_os = "windows")]
  #[error("permission denied when trying to read NotITG memory: {0}\n\
           since there is currently a lack of Windows testing, i (as an error message) cannot offer help as to how to fix this. sorry!")]
  MemoryPermissionError(#[source] std::io::Error),
  #[cfg(target_os = "macos")]
  #[error("permission denied when trying to read NotITG memory: {0}\n\
           since there is currently a lack of macOS testing, i (as an error message) cannot offer help as to how to fix this. sorry!")]
  MemoryPermissionError(#[source] std::io::Error),
  #[cfg(target_os = "linux")]
  #[error("permission denied when trying to read NotITG memory: {0}\n\
           this may indicate that the process is running under a different UID, or that your OS has unprivileged debugging enabled\n\
           try running `sudo sysctl -w kernel.yama.ptrace_scope=0` to temporarily lift the memory read restrictions globally on your system")]
  MemoryPermissionError(#[source] std::io::Error),

  #[error("error reading from NotITG process: {0}")]
  MemoryReadError(#[source] std::io::Error),
  #[error("error writing to NotITG process: {0}")]
  MemoryWriteError(#[source] std::io::Error),

  #[error("could not find compatible NotITG process")]
  NoCompatibleProcessFoundError,
  #[error("no connected NotITG process")]
  NotConnectedError,
  #[error("cannot open connection while a connection is still open")]
  ConnectionStillOpenError,
}

impl Serialize for NotITGError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

pub type Slot = i32;

#[derive(Clone, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
  // the start of the external area
  pub base_address: usize,
  // the size of the external area, in Slots
  pub size: usize,

  // where an 8-byte build string is located
  pub build_address: usize,
  // the expected 8-byte build string
  pub build_string: &'static str,
}

// borrowed mostly from https://github.com/Jaezmien/NotITG-External-Python/blob/main/notitg.py
pub static NOTITG_VERSIONS: LazyLock<HashMap<&str, VersionInfo>> = LazyLock::new(|| {
  HashMap::from([
    ("4.2", VersionInfo {
      base_address: 0x008BFF38,
      size: 256,
      build_address: 0x006FAD40,
      build_string: "20210420",
    }),
    ("4.9", VersionInfo {
      base_address: 0x00AA6D78,
      size: 256,
      build_address: 0x0099A318,
      build_string: "20240917",
    }),
    ("4.9.1", VersionInfo {
      base_address: 0x00AA6D98,
      size: 256,
      build_address: 0x0099A318,
      build_string: "20241010",
    }),
  ])
});

pub fn read_addr<T: Copy>(handle: ProcessHandle, addr: usize) -> std::io::Result<T> {
  let member: DataMember<T> = DataMember::new_addr(handle, addr);
  unsafe { member.read() }
}

pub fn read_addr_vec<T: Copy>(handle: ProcessHandle, addr: usize, size: usize) -> std::io::Result<Vec<T>> {
  Ok(copy_address(addr, size * std::mem::size_of::<T>(), &handle)?
    .chunks_exact(std::mem::size_of::<T>())
    .map(|chunk| unsafe { std::ptr::read_unaligned(chunk.as_ptr() as *const T) })
    .collect())
}

pub fn write_addr<T: Copy>(handle: ProcessHandle, addr: usize, value: &T) -> std::io::Result<()> {
  let member: DataMember<T> = DataMember::new_addr(handle, addr);
  member.write(value)
}

pub fn identify_notitg_version(handle: ProcessHandle) -> Result<Option<&'static str>, NotITGError> {
  for (ver, info) in NOTITG_VERSIONS.iter() {
    info!("trying ver {ver} ({}); reading addr {:#x}", info.build_string, info.build_address);
    let build: [u8; 8] = match read_addr(handle, info.build_address) {
      Ok(arr) => arr,
      Err(err) if err.kind() == ErrorKind::PermissionDenied => {
        return Err(NotITGError::MemoryPermissionError(err));
      },
      Err(err) => {
        info!("{:#?}", err);
        continue;
      }
    };

    let s = std::str::from_utf8(&build).unwrap_or_default();

    info!("got {s}");

    if s != info.build_string {
      continue
    }

    info!("build number matches yay!!!");

    return Ok(Some(ver));
  }

  Ok(None)
}

pub fn find_notitg_pid() -> Result<(Pid, &'static str), NotITGError> {
  let mut s = System::new();
  s.refresh_processes_specifics(
    sysinfo::ProcessesToUpdate::All,
    true,
    sysinfo::ProcessRefreshKind::nothing()
      .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
  );

  for (pid, process) in s.processes() {
    let cmd = process.cmd();

    let has_match = cmd
      .iter()
      .filter_map(|part| part.to_str())
      .any(|part| regex!(r"NotITG.+\.exe$").is_match(part.trim()));

    if !has_match {
      continue;
    }
    
    info!("found candidate pid {pid} ({:?})", cmd);

    let Ok(handle) = (pid.as_u32() as Pid).try_into_process_handle() else {
      info!("{pid}: failed to create process handle");
      continue;
    };
    return match identify_notitg_version(handle) {
      Err(err) => {
        return Err(err);
      },
      Ok(None) => {
        info!("{pid}: failed to find matching NotITG version");
        continue;
      },
      Ok(Some(ver)) => Ok((handle.0 as _, ver))
    };
  }

  Err(NotITGError::NoCompatibleProcessFoundError)
}
