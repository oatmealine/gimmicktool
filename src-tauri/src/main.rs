// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use process_memory::{DataMember, Memory, Pid, ProcessHandle, TryIntoProcessHandle, copy_address};
use std::{collections::HashMap};
use std::io::ErrorKind;
use std::sync::LazyLock;
use regex::regex;
use sysinfo::System;
//use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt;

#[derive(Debug, Clone, serde::Serialize)]
struct NotITGProcessError {
  message: Option<&'static str>,
}

impl fmt::Display for NotITGProcessError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.message {
      Some(s) => write!(f, "could not communicate with the NotITG process: {}", s),
      None => write!(f, "could not communicate with the NotITG process")
    }
  }
}

#[derive(Clone, Copy, serde::Serialize)]
struct VersionInfo {
  // the start of the external area
  base_address: usize,
  // the size of the external area, in u32s
  size: usize,

  // where an 8-byte build string is located
  build_address: usize,
  // the expected 8-byte build string
  build_string: &'static str,
}

/*impl Serialize for VersionInfo {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut state = serializer.serialize_struct("VersionInfo", 4)?;
    state.serialize_field("base_address", &self.base_address)?;
    state.serialize_field("size", &self.size)?;
    state.serialize_field("build_address", &self.build_address)?;
    state.serialize_field("build_string", &self.build_string)?;
    state.end()
  }
}*/

// borrowed mostly from https://github.com/Jaezmien/NotITG-External-Python/blob/main/notitg.py
static NOTITG_VERSIONS: LazyLock<HashMap<&str, VersionInfo>> = LazyLock::new(|| {
  return HashMap::from([
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
  ]);
});

fn read_addr<T: Copy>(handle: ProcessHandle, addr: usize) -> std::io::Result<T> {
  let member: DataMember<T> = DataMember::new_offset(handle, vec![addr]);
  unsafe { member.read() }
}

fn read_addr_vec<T: Copy>(handle: ProcessHandle, addr: usize, size: usize) -> std::io::Result<Vec<T>> {
  Ok(copy_address(addr, size * std::mem::size_of::<T>(), &handle)?
    .chunks_exact(std::mem::size_of::<T>())
    .map(|chunk| unsafe { std::ptr::read_unaligned(chunk.as_ptr() as *const T) })
    .collect())
}

fn write_addr<T: Copy>(handle: ProcessHandle, addr: usize, value: &T) -> std::io::Result<()> {
  let member: DataMember<T> = DataMember::new_offset(handle, vec![addr]);
  member.write(value)
}

fn identify_notitg_version(handle: ProcessHandle) -> Result<Option<&'static str>, NotITGProcessError> {
  let pid = handle.0;
  for (ver, info) in NOTITG_VERSIONS.iter() {
    println!("{pid}: trying ver {ver} ({}); trying addr {:#x}", info.build_string, info.build_address);
    let build: [u8; 8] = match read_addr(handle, info.build_address as usize) {
      Ok(arr) => arr,
      Err(err) => {
        if err.kind() == ErrorKind::PermissionDenied {
          return Err(NotITGProcessError {
            message: Some("permission denied when trying to read address. this may indicate that the process is running under a different UID")
          });
        }
    
        continue;
        //return NotITGProcessError { message: format!("{:#?}", err) };
      }
    };

    let s = std::str::from_utf8(&build).unwrap_or_default();

    println!("{pid}: got {s}");

    if s != info.build_string {
      continue
    }

    println!("{pid}: build number matches yay!!!");

    return Ok(Some(ver));
  }

  Ok(None)
}

fn find_notitg_pid() -> Result<(ProcessHandle, &'static str), NotITGProcessError> {
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
      .map(|part| part.to_str())
      .flatten()
      .any(|part| regex!(r"NotITG.+\.exe$").is_match(part.trim()));

    if !has_match {
      continue;
    }
    
    println!("found candidate pid {pid} ({:?})", cmd);

    let Ok(handle) = (pid.as_u32() as Pid).try_into_process_handle() else {
      println!("{pid}: failed to create process handle");
      continue;
    };
    return match identify_notitg_version(handle) {
      Err(err) => {
        return Err(err);
      },
      Ok(None) => {
        println!("{pid}: failed to find matching NotITG version");
        continue;
      },
      Ok(Some(ver)) => Ok((handle, ver))
    };
  }

  Err(NotITGProcessError { message: Some("no compatible NotITG process found") })
}

#[tauri::command]
fn try_connect() -> Result<(Pid, &'static str, VersionInfo), NotITGProcessError> {
  let (handle, ver) = find_notitg_pid()?;
  Ok((handle.0, ver, NOTITG_VERSIONS[ver]))
}

#[tauri::command]
fn get_external(pid: Pid, base_address: usize, size: usize) -> Result<Vec<i32>, String> {
  let handle = pid.try_into_process_handle()
    .map_err(|e| e.to_string())?;
  let externals: Vec<i32> = read_addr_vec(handle, base_address, size)
    .map_err(|e| e.to_string())?;
  Ok(externals)
}

fn main() {
  //let externals: Vec<i32> = read_addr_vec(handle, ver_info.base_address, ver_info.size);

  //for (i, v) in externals.iter().enumerate() {
  //  println!("externals[{i}] = {}", v);
  //}

  //write_addr(handle, ver_info.base_address, &102u32);
  //println!("wrote some buuuullshit");

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![try_connect, get_external])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}