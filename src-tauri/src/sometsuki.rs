use std::time::{Duration, Instant};

use log::{debug, error, info, trace, warn};
use process_memory::{Pid, ProcessHandle, TryIntoProcessHandle};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::ipc::Channel;

use crate::notitg::{NotITGError, Slot, read_addr, read_addr_vec, write_addr};

// headers
const H2C_ACK: Slot = 0x01;
const H2C_WRITING: Slot = 0x02;
const H2C_READY: Slot = 0x03;
const C2H_ACK: Slot = 0x04;
const C2H_WRITING: Slot = 0x05;
const C2H_READY: Slot = 0x06;

// special bytes
const STREAM_END: i32 = 0x00;
const STREAM_SEP: i32 = 0x01;

struct SometsukiConnection {
  handle: ProcessHandle,
  base_address: usize,
  size: usize,
  
  write_buf: Vec<Slot>,
  read_buf: Vec<Slot>,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
pub enum ConnectionEvent {
  Connected { name: String, version: String },
  Message { value: Value },
  Error { message: String },
  Disconnected,
}

pub enum SometsukiCommand {
  Connect {
    reply: oneshot::Sender<Result<(), NotITGError>>,

    pid: Pid, base_address: usize, size: usize,
    channel: Channel<ConnectionEvent>,
  },
  SendMessage {
    reply: oneshot::Sender<Result<(), NotITGError>>,

    value: Value,
  },
  Disconnect,
}

pub struct Sometsuki {
  conn: Option<SometsukiConnection>,
  /// communicates back to the frontend
  channel: Option<Channel<ConnectionEvent>>,

  /// turns 'true' when a 'hello' is sent from the host
  opened: bool,
  
  last_header: Slot,
  hold_started: Instant,
  last_message_received: Instant,
}

impl SometsukiConnection {
  fn write_header(&mut self, value: &Slot) -> std::io::Result<()> {
    write_addr(self.handle, self.base_address, value)
  }
  fn read_header(&mut self) -> std::io::Result<Slot> {
    read_addr(self.handle, self.base_address)
  }

  fn flush_read_buffer(&mut self) -> Vec<Slot> {
    std::mem::take(&mut self.read_buf)
  }

  pub fn read(&mut self) -> std::io::Result<Vec<Vec<Slot>>> {
    let mut msgs: Vec<Vec<Slot>> = Vec::new();

    let buffer: Vec<Slot> = read_addr_vec(
      self.handle,
      self.base_address + size_of::<Slot>(),
      self.size - 1
    )?;

    for v in buffer {
      match v {
        STREAM_SEP => msgs.push(self.flush_read_buffer()),
        STREAM_END => {
          msgs.push(self.flush_read_buffer());
          break;
        },
        _ => self.read_buf.push(v),
      };
    }
    
    Ok(msgs)
  }

  pub fn write(&mut self, should_write_ack: bool) -> std::io::Result<()> {
    if !self.write_buf.is_empty() {
      self.write_header(&C2H_WRITING)?;
      // TODO: this is a mess
      for (i, value) in self.write_buf.drain(0..(self.size - 1).min(self.write_buf.len())).enumerate() {
        write_addr(
          self.handle, 
          self.base_address + (i + 1) * size_of::<Slot>(),
          &value
        )?;
      }
      self.write_header(&C2H_READY)?;
      Ok(())
    } else if should_write_ack {
      self.write_header(&C2H_ACK)
    } else {
      Ok(())
    }
  }

  pub fn flush_write_buffer(&mut self) -> std::io::Result<()> {
    self.write(false)
  }
  
  fn send_message(&mut self, msg: Vec<Slot>) {
    if !self.write_buf.is_empty() {
      // replace the end STREAM_END with STREAM_SEP
      self.write_buf.pop();
      self.write_buf.push(STREAM_SEP);
    }
    self.write_buf.extend(msg);
    self.write_buf.push(STREAM_END);
  }
}

fn msg_encode(val: &Value) -> Result<Vec<Slot>, NotITGError> {
  let json = serde_json::to_string(val)?;
  let encoded = json.into_bytes().into_iter().map(|b| b as i32).collect();
  Ok(encoded)
}

fn msg_decode(data: &[Slot]) -> Result<Value, NotITGError> {
  let bytes: Vec<u8> = data.iter().map(|b| *b as u8).collect();
  let decoded = str::from_utf8(&bytes)?;
  let data: Value = serde_json::from_str(decoded)?;
  Ok(data)
}

impl Sometsuki {
  pub fn new() -> Sometsuki {
    Sometsuki {
      conn: None,
      channel: None,

      opened: false,

      last_header: 0x00,
      hold_started: Instant::now(),
      last_message_received: Instant::now(),
    }
  }

  pub fn connect(&mut self, pid: Pid, base_address: usize, size: usize, channel: Channel<ConnectionEvent>) -> Result<(), NotITGError> {
    if self.conn.is_some() {
      return Err(NotITGError::ConnectionStillOpenError);
    }

    let conn = SometsukiConnection {
      handle: pid.try_into_process_handle()?,
      base_address,
      size,

      write_buf: Vec::new(),
      read_buf: Vec::new(),
    };

    self.conn = Some(conn);
    self.channel = Some(channel);

    self.opened = false;
    self.last_header = 0x00;
    self.hold_started = Instant::now();
    self.last_message_received = Instant::now();

    self.greet()?;

    Ok(())
  }

  pub fn is_closed(&mut self) -> bool {
    self.conn.is_none()
  }
  
  /// _does not immediately send the message_; it will be stored in the write
  /// buffer until next possible write opportunity
  pub fn send_message(&mut self, msg: &Value) -> Result<(), NotITGError> {
    if self.is_closed() {
      return Err(NotITGError::NotConnectedError);
    }

    trace!("> {msg}");
    self.conn.as_mut().unwrap().send_message(msg_encode(msg)?);
    Ok(())
  }
  /// sends a message and immediately writes it, disregarding the current memory
  /// contents
  fn send_message_force(&mut self, msg: &Value) -> Result<(), NotITGError> {
    self.send_message(msg)?;
    self.conn.as_mut().unwrap().flush_write_buffer()
      .map_err(NotITGError::MemoryWriteError)
  }

  fn on_message(&mut self, msg: Value) {
    trace!("< {msg}");

    let Some(obj) = msg.as_object() else {
      self.emit_error(&format!("expected object, got {}", msg));
      return;
    };
    let Some(msg_type) = obj.get("t") else {
      self.emit_error("message lacks 't' field");
      return;
    };
    let Some(msg_type) = msg_type.as_str() else {
      self.emit_error("'t' is not a string");
      return;
    };

    match msg_type {
      "heartbeat" => (),
      "hello" => {
        if self.opened {
          self.emit_error("duplicate 'hello' message");
          self.disconnect(false);
        } else {
          info!("hello recieved; connection now open");
          self.opened = true;

          let Some(name) = obj.get("n") else {
            self.emit_error("'hello' message lacks 'n' field");
            self.disconnect(false);
            return;
          };
          let Some(name) = name.as_str() else {
            self.emit_error("'hello' message 'n' is not string");
            self.disconnect(false);
            return;
          };
          let Some(version) = obj.get("v") else {
            self.emit_error("'hello' message lacks 'v' field");
            self.disconnect(false);
            return;
          };
          let Some(version) = version.as_str() else {
            self.emit_error("'hello' message 'v' is not string");
            self.disconnect(false);
            return;
          };

          self.channel.as_ref().unwrap().send(ConnectionEvent::Connected {
            name: name.to_owned(), version: version.to_owned()
          }).unwrap();
        }
      },
      "goodbye" => {
        debug!("goodbye received, closing connection :(");
        self.disconnect(true);
      },
      "error" => {
        let Some(err) = obj.get("m") else {
          self.emit_error("'error' message lacks 'm' field");
          return;
        };
        let Some(err) = err.as_str() else {
          self.emit_error("'error' message 'm' is not a string");
          return;
        };

        warn!("{err}");
        self.channel.as_ref().unwrap().send(ConnectionEvent::Error { message: err.to_owned() }).unwrap();
      },
      _ => {
        if self.opened {
          self.channel.as_ref().unwrap().send(ConnectionEvent::Message { value: msg }).unwrap();
        }
      }
    }
  }
  
  fn on_message_raw(&mut self, msg: Vec<Slot>) {
    match msg_decode(&msg) {
      Ok(msg) => self.on_message(msg),
      Err(e) => {
        warn!("< [malformed data]");
        self.emit_error(&e.to_string())
      }
    }
  }
  
  fn hold(&mut self) {
    trace!("holding");
    if self.hold_started.elapsed() > Sometsuki::HOLD_DURATION {
      // consider connection dropped
      info!("connection timed out");
      self.channel.as_ref().unwrap().send(ConnectionEvent::Error { message: "connection timeout".to_string() }).unwrap();
      self.disconnect(false);
    }
  }

  pub fn try_process(&mut self) {
    match self.process() {
      Ok(()) => (),
      Err(e) => {
        error!("error while processing: {e}");
        self.channel.as_ref().unwrap().send(ConnectionEvent::Error { message: e.to_string() }).unwrap();
        self.disconnect(true);
      },
    }
  }

  fn process(&mut self) -> Result<(), NotITGError> {
    assert!(!self.is_closed(), "process() called while a connection is not active");

    let header = self.conn.as_mut().unwrap().read_header()
      .map_err(NotITGError::MemoryReadError)?;
    let is_new_header = header != self.last_header;
    self.last_header = header;

    match header {
      // host has sent an ACK, we can continue writing if we have anything to write
      H2C_ACK => {
        if is_new_header {
          self.last_message_received = Instant::now();
        }
        self.conn.as_mut().unwrap().write(false)
          .map_err(NotITGError::MemoryWriteError)
      },
      // host has not sent a message since, we can write if we have anything to write
      C2H_ACK => self.conn.as_mut().unwrap().write(false)
        .map_err(NotITGError::MemoryWriteError),
      // host is not yet done writing, hold
      // or host has not yet read our message, hold
      H2C_WRITING | C2H_READY => {
        if is_new_header {
          self.hold_started = Instant::now();
        }
        self.hold();
        Ok(())
      },
      // host is done writing, read response then write back
      H2C_READY => {
        self.last_message_received = Instant::now();

        let msgs = self.conn.as_mut().unwrap().read()
          .map_err(NotITGError::MemoryReadError)?;
        for msg in msgs {
          self.on_message_raw(msg);
        }

        self.conn.as_mut().unwrap().write(true)
          .map_err(NotITGError::MemoryWriteError)
      },
      // probably writing on seperate thread(?!), hold
      C2H_WRITING => Ok(()),

      // invalid header
      _ => self.conn.as_mut().unwrap().write(true)
        .map_err(NotITGError::MemoryWriteError),
    }?;

    if self.last_message_received.elapsed() > Sometsuki::HEARTBEAT_INTERVAL {
      self.emit_heartbeat();
      self.last_message_received = Instant::now();
    }

    Ok(())
  }

  const HOLD_DURATION: Duration = Duration::from_secs(5);
  const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

  const NAME: &'static str = "gimmicktool";
  const VERSION: &'static str = "0.0.0";

  fn greet(&mut self) -> Result<(), NotITGError> {
    debug!("sending hello to host");
    self.send_message_force(&json!({
      "t": "hello",
      "n": Sometsuki::NAME,
      "v": Sometsuki::VERSION,
    }))
  }

  fn emit_error(&mut self, msg: &str) {
    debug!("sending error to host: {msg}");
    match self.send_message(&json!({
      "t": "error",
      "m": msg
    })) {
      Ok(_) => (),
      Err(e) => {
        error!("failed sending error to host: {e}\n\
                                     assuming irrecoverable and disconnecting");
        self.disconnect(false)
      }
    }
  }

  fn emit_heartbeat(&mut self) {
    match self.send_message(&json!({
      "t": "heartbeat"
    })) {
      Ok(_) => (),
      Err(e) => {
        error!("failed sending heartbeat: {e}");
        self.disconnect(true)
      }
    };
  }

  pub fn disconnect(&mut self, quiet: bool) {
    if self.is_closed() {
      warn!("attempted to disconnect while not connected, ignoring");
      return;
    }

    self.channel.as_ref().unwrap().send(ConnectionEvent::Disconnected).unwrap();
    info!("disconnecting");
    if !quiet {
      match self.send_message_force(&json!({
        "t": "goodbye"
      })) {
        Ok(_) => (),
        Err(e) => {
          warn!("failed sending goodbye to host: {e}\n\
                                      not really our issue anymore, ignoring");
        }
      };
    }
    self.conn = None;
    self.channel = None;
  }
}
