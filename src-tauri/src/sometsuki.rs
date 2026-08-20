use std::time::{Duration, Instant};

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

pub struct Sometsuki {
  handle: ProcessHandle,
  base_address: usize,
  size: usize,
  
  write_buf: Vec<Slot>,
  read_buf: Vec<Slot>,

  // turns `true` when a `hello` is sent from the host
  opened: bool,
  closed: bool,

  hold_started: Instant,
  last_message_received: Instant,

  channel: Channel<ConnectionEvent>,
  last_header: Slot,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
pub enum ConnectionEvent {
  Connected { name: String, version: String },
  Message { value: Value },
  Error { message: String },
  Disconnected,
}

impl Sometsuki {
  fn msg_encode(val: &Value) -> Result<Vec<Slot>, NotITGError> {
    let json = serde_json::to_string(val)?;
    let encoded = json.as_bytes().iter().map(|b| *b as i32).collect();
    Ok(encoded)
  }

  fn msg_decode(data: &[Slot]) -> Result<Value, NotITGError> {
    let bytes: Vec<u8> = data.iter().map(|b| *b as u8).collect();
    let decoded = str::from_utf8(&bytes)?;
    let data: Value = serde_json::from_str(decoded)?;
    Ok(data)
  }

  pub fn new(pid: Pid, base_address: usize, size: usize, channel: Channel<ConnectionEvent>) -> Result<Sometsuki, NotITGError> {
    let mut sometsuki = Sometsuki {
      handle: pid.try_into_process_handle()?,
      base_address,
      size,

      write_buf: Vec::new(),
      read_buf: Vec::new(),

      opened: false,
      closed: false,

      hold_started: Instant::now(),
      last_message_received: Instant::now(),

      channel,

      last_header: 0,
    };

    sometsuki.greet()?;

    Ok(sometsuki)
  }

  pub fn is_closed(&mut self) -> bool {
    self.closed
  }
  
  fn send_message_raw(&mut self, msg: Vec<Slot>) {
    if !self.write_buf.is_empty() {
      // replace the end STREAM_END with STREAM_SEP
      self.write_buf.pop();
      self.write_buf.push(STREAM_SEP);
    }
    self.write_buf.extend(msg);
    self.write_buf.push(STREAM_END);
  }
  pub fn send_message(&mut self, msg: &Value) -> Result<(), NotITGError> {
    if self.closed {
      return Err(NotITGError::ConnectionClosedError);
    }
    self.log(&format!("> {msg}"));
    self.send_message_raw(Sometsuki::msg_encode(msg)?);
    Ok(())
  }
  fn send_message_force(&mut self, msg: &Value) -> Result<(), NotITGError> {
    self.send_message(msg)?;
    self.write(false)
  }

  fn on_message(&mut self, msg: Value) {
    self.log(&format!("< {msg}"));

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
          self.log("hello recieved; connection now open");
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

          self.channel.send(ConnectionEvent::Connected {
            name: name.to_owned(), version: version.to_owned()
          }).unwrap();
        }
      },
      "goodbye" => {
        self.log("goodbye received, closing connection :(");
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

        self.log(&format!("ERR: {err}"));
        self.channel.send(ConnectionEvent::Error { message: err.to_owned() }).unwrap();
      },
      _ => {
        if self.opened {
          self.channel.send(ConnectionEvent::Message { value: msg }).unwrap();
        }
      }
    }
  }

  fn on_message_raw(&mut self, msg: Vec<Slot>) {
    match Sometsuki::msg_decode(&msg) {
      Ok(msg) => self.on_message(msg),
      Err(e) => {
        self.log("< [malformed data]");
        self.emit_error(&e.to_string())
      }
    }
  }

  fn write_header(&mut self, value: &Slot) -> Result<(), NotITGError> {
    write_addr(self.handle, self.base_address, value)
      .map_err(|e| NotITGError::MemoryWriteError { source: e })
  }
  fn read_header(&mut self) -> Result<Slot, NotITGError> {
    read_addr(self.handle, self.base_address)
      .map_err(|e| NotITGError::MemoryReadError { source: e })
  }

  fn flush_message(&mut self) {
    let msg = std::mem::take(&mut self.read_buf);
    self.on_message_raw(msg);
  }

  fn read(&mut self) -> Result<(), NotITGError> {
    let buffer: Vec<Slot> = read_addr_vec(
      self.handle,
      self.base_address + size_of::<Slot>(),
      self.size - 1
    )
      .map_err(|e| NotITGError::MemoryReadError { source: e })?;

    for v in buffer {
      match v {
        STREAM_SEP => self.flush_message(),
        STREAM_END => {
          self.flush_message();
          break;
        },
        _ => self.read_buf.push(v),
      };
    }
    
    Ok(())
  }

  fn write(&mut self, should_write_ack: bool) -> Result<(), NotITGError> {
    if !self.write_buf.is_empty() {
      self.write_header(&C2H_WRITING)?;
      // TODO: this is a mess
      for (i, value) in self.write_buf.drain(0..(self.size.min(self.write_buf.len() + 1)) - 1).enumerate() {
        write_addr(
          self.handle, 
          self.base_address + (i + 1) * size_of::<Slot>(),
          &value
        )
          .map_err(|e| NotITGError::MemoryWriteError { source: e })?;
      }
      self.write_header(&C2H_READY)?;
      Ok(())
    } else if should_write_ack {
      self.write_header(&C2H_ACK)
    } else {
      Ok(())
    }
  }

  const HOLD_DURATION: Duration = Duration::from_secs(5);
  const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

  fn hold(&mut self) {
    self.log("holding");
    if self.hold_started.elapsed() > Sometsuki::HOLD_DURATION {
      // consider connection dropped
      self.disconnect(false);
    }
  }

  pub fn process(&mut self) -> Result<(), NotITGError> {
    if self.closed {
      return Err(NotITGError::ConnectionClosedError);
    }

    let header = self.read_header()?;
    let is_new_header = header != self.last_header;
    self.last_header = header;

    match header {
      // host has sent an ACK, we can continue writing if we have anything to write
      H2C_ACK => {
        if is_new_header {
          self.last_message_received = Instant::now();
        }
        self.write(false)
      },
      // host has not sent a message since, we can write if we have anything to write
      C2H_ACK => self.write(false),
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
        self.read()?;
        self.write(true)
      },
      // probably writing on seperate thread(?!), hold
      C2H_WRITING => Ok(()),

      // invalid header
      _ => self.write(true),
    }?;

    if self.last_message_received.elapsed() > Sometsuki::HEARTBEAT_INTERVAL {
      self.emit_heartbeat();
      self.last_message_received = Instant::now();
    }

    Ok(())
  }
  
  const NAME: &'static str = "gimmicktool";
  const VERSION: &'static str = "0.0.0";

  fn greet(&mut self) -> Result<(), NotITGError> {
    self.log("sending hello to host");
    self.send_message_force(&json!({
      "t": "hello",
      "n": Sometsuki::NAME,
      "v": Sometsuki::VERSION,
    }))
  }

  pub fn emit_error(&mut self, msg: &str) {
    self.log(&format!("sometsuki: sending error to host: {msg}"));
    match self.send_message(&json!({
      "t": "error",
      "m": msg
    })) {
      Ok(_) => (),
      Err(e) => {
        self.log(&format!("failed sending error to host: {e}"));
        self.log("sometsuki: assuming irrecoverable and disconnecting");
        self.disconnect(false)
      }
    }
  }

  fn emit_heartbeat(&mut self) {
    let _ = self.send_message(&json!({
      "t": "heartbeat"
    }));
  }

  pub fn disconnect(&mut self, quiet: bool) {
    self.channel.send(ConnectionEvent::Disconnected).unwrap();
    self.log("disconnecting");
    if !quiet {
      match self.send_message_force(&json!({
        "t": "goodbye"
      })) {
        Ok(_) => (),
        Err(e) => {
          self.log(&format!("failed sending goodbye to host: {e}"));
          self.log("not really our issue anymore, ignoring");
        }
      };
    }
    self.closed = true;
  }

  pub fn log(& self, msg: &str) {
    println!("[sometsuki<{}>] {}", self.handle.0, msg)
  }
}