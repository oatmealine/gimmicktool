use std::collections::VecDeque;
use std::fmt;
use std::{fmt::Display, fmt::Debug};

use serde::{Serialize, Deserialize, forward_to_deserialize_any, ser};
use serde::de::{
  self, DeserializeSeed, MapAccess, SeqAccess, Visitor
};
use serde_json::Value;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
  #[error("integer exceeds maximum size")]
  IntegerExceedsMaxSize,
  #[error("string exceeds maximum size")]
  StringExceedsMaxSize,
  #[error("sequence exceeds maximum size")]
  SequenceExceedsMaxSize,
  #[error("map exceeds maximum size")]
  MapExceedsMaxSize,
  #[error("sequence/map requires length to be known up front")]
  LengthRequired,
  #[error("cannot use nil as table index")]
  CannotUseNilAsIndex, // TODO not implemented

  #[error("junk data at end of bytes")]
  TrailingCharacters,
  #[error("unexpected end of data")]
  Eof,
  #[error("unknown value type {0}")]
  UnknownType(u8),
  #[error("UTF8 decoding error: {0}")]
  Utf8(#[from] std::string::FromUtf8Error),
  
  #[error("{0}")]
  Custom(String),
}

impl ser::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
    Error::Custom(msg.to_string())
  }
}
impl de::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
    Error::Custom(msg.to_string())
  }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Default)]
pub enum Message {
  #[default]
  Nil,
  Int(i32),
  Float(f64),
  Boolean(bool),
  String(String), // probably make this some utf-8 non-requiring type? idk
  Array(Array),
  Map(Map),
}
pub type Array = Vec<Message>;

#[derive(Clone, Debug, Default)]
pub struct Map {
  pub inner: Vec<(Message, Message)>
}
impl Map {
  pub fn new() -> Map {
    Map { inner: Vec::new() }
  }
  pub fn from(vec: Vec<(Message, Message)>) -> Map {
    Map { inner: vec }
  }

  pub fn get(&self, index: &str) -> Option<&Message> {
    self.inner.iter()
      .find(|n| match n.0 {
        Message::String(ref s) => s == index,
        _ => false,
      })
      .map(|n| &n.1)
  }
}

impl Message {
  pub fn from_bytes(bytes: &[u8]) -> Result<Message> {
    let mut deserializer = Deserializer::from_bytes(bytes);
    Message::deserialize(&mut deserializer)
  }
  pub fn from_json(value: &Value) -> Result<Message> {
    Ok(match *value {
      Value::Null => Message::Nil,
      Value::Number(ref n) => match n {
        n if n.as_i64().is_some() => match i32::try_from(n.as_i64().unwrap()) {
          Ok(i) => Message::Int(i),
          Err(_) => Message::Float(n.as_f64().unwrap()), // just hope this doesn't fail. lol
        }
        n if n.as_f64().is_some() => Message::Float(n.as_f64().unwrap()),
        _ => return Err(Error::IntegerExceedsMaxSize),
      },
      Value::Bool(b) => Message::Boolean(b),
      Value::String(ref s) => Message::String(s.clone()),
      Value::Array(ref a) =>
        Message::Array(
          a.iter()
            .map(|v| Message::from_json(v))
            .collect::<Result<Vec<Message>>>()?
        ),
      Value::Object(ref map) =>
        Message::Map(Map::from(
          map.iter()
            .map(|(k, v)| Ok((Message::String(k.clone()), Message::from_json(v)?)))
            .collect::<Result<Vec<(Message, Message)>>>()?
        ))
    })
  }

  pub fn as_nil(&self) -> Option<()> {
    match *self {
      Message::Nil => Some(()),
      _ => None,
    }
  }
  pub fn as_int(&self) -> Option<i32> {
    match *self {
      Message::Int(v) => Some(v),
      _ => None,
    }
  }
  pub fn as_float(&self) -> Option<f64> {
    match *self {
      Message::Float(v) => Some(v),
      _ => None,
    }
  }
  pub fn as_bool(&self) -> Option<bool> {
    match *self {
      Message::Boolean(v) => Some(v),
      _ => None,
    }
  }
  pub fn as_str(&self) -> Option<&str> {
    match *self {
      Message::String(ref v) => Some(v.as_str()),
      _ => None,
    }
  }
  pub fn as_string(&self) -> Option<&String> {
    match *self {
      Message::String(ref v) => Some(v),
      _ => None,
    }
  }
  pub fn as_array(&self) -> Option<&Array> {
    match *self {
      Message::Array(ref v) => Some(v),
      _ => None,
    }
  }
  pub fn as_map(&self) -> Option<&Map> {
    match *self {
      Message::Map(ref v) => Some(v),
      _ => None,
    }
  }
}

impl Display for Message {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Message::Nil => write!(fmt, "nil"),
      Message::Int(i) => write!(fmt, "{}", i),
      Message::Float(f) => write!(fmt, "{}", f),
      Message::Boolean(b) => write!(fmt, "{}", b),
      Message::String(ref s) => write!(fmt, "\"{}\"", s),
      Message::Array(ref s) => {
        fmt.write_str("[")?;
        let mut first = true;
        for message in s {
          if !first {
            fmt.write_str(",")?;
          }
          first = false;
          write!(fmt, "{}", message)?;
        }
        fmt.write_str("]")
      },
      Message::Map(ref s) => {
        fmt.write_str("{")?;
        let mut first = true;
        for (key, value) in &s.inner {
          if !first {
            fmt.write_str(",")?;
          }
          first = false;
          write!(fmt, "{}:{}", key, value)?;
        }
        fmt.write_str("}")
      },
    }
  }
}

pub struct Serializer {
  output: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
  T: Serialize,
{
  let mut serializer = Serializer {
    output: Vec::new(),
  };
  value.serialize(&mut serializer)?;
  Ok(serializer.output)
}

impl Serializer {
  // for the common case of the same type having multiple types for different
  // sizes; fits the length to the smallest type it can be and then sets the
  // type accordingly
  fn serialize_len(
    &mut self, len: usize, case_u8: u8, case_u16: u8, case_u32: u8
  ) -> Option<()> {
    #[allow(clippy::match_overlapping_arm)]
    match len {
      ..=0xFF => {
        self.output.push(case_u8);
        self.output.extend((len as u8).to_be_bytes());
      }
      ..=0xFFFF => {
        self.output.push(case_u16);
        self.output.extend((len as u16).to_be_bytes());
      }
      ..=0xFFFFFFFF => {
        self.output.push(case_u32);
        self.output.extend((len as u32).to_be_bytes());
      }
      _ => return None,
    }
    Some(())
  }
}

impl ser::Serializer for &mut Serializer {
  type Ok = ();

  type Error = Error;

  type SerializeSeq = Self;
  type SerializeTuple = Self;
  type SerializeTupleStruct = Self;
  type SerializeTupleVariant = Self;
  type SerializeMap = Self;
  type SerializeStruct = Self;
  type SerializeStructVariant = Self;

  fn serialize_i8(self, v: i8) -> Result<()> {
    self.output.push(0x01);
    self.output.push(v.cast_unsigned());
    Ok(())
  }

  fn serialize_i16(self, v: i16) -> Result<()> {
    match i8::try_from(v) {
      Ok(n) => self.serialize_i8(n),
      Err(_) => {
        self.output.push(0x02);
        self.output.extend(v.to_be_bytes());
        Ok(())
      }
    }
  }

  fn serialize_i32(self, v: i32) -> Result<()> {
    match i16::try_from(v) {
      Ok(n) => self.serialize_i16(n),
      Err(_) => {
        self.output.push(0x03);
        self.output.extend(v.to_be_bytes());
        Ok(())
      }
    }
  }

  fn serialize_i64(self, v: i64) -> Result<()> {
    match i32::try_from(v) {
      Ok(n) => self.serialize_i32(n),
      Err(_) => self.serialize_f64(v as f64),
    }
  }

  fn serialize_u8(self, v: u8) -> Result<()> {
    self.serialize_i16(i16::from(v))
  }

  fn serialize_u16(self, v: u16) -> Result<()> {
    self.serialize_i32(i32::from(v))
  }

  fn serialize_u32(self, v: u32) -> Result<()> {
    self.serialize_i64(i64::from(v))
  }

  fn serialize_u64(self, v: u64) -> Result<()> {
    match i32::try_from(v) {
      Ok(n) => self.serialize_i32(n),
      // this could be lossy, but should hopefully never be
      Err(_) => self.serialize_f64(v as f64),
    }
  }

  fn serialize_f32(self, v: f32) -> Result<()> {
    self.serialize_f64(f64::from(v))
  }

  fn serialize_f64(self, v: f64) -> Result<()> {
    self.output.push(0x04);
    self.output.extend(v.to_be_bytes());
    Ok(())
  }
  
  fn serialize_bool(self, v: bool) -> Result<()> {
    self.output.push(if v { 0x05 } else { 0x06 });
    Ok(())
  }

  fn serialize_char(self, v: char) -> Result<()> {
    match u8::try_from(v) {
      Ok(n) => {
        self.output.push(0x07);
        self.output.push(n);
        Ok(())
      },
      Err(_) => self.serialize_str(&v.to_string())
    }
  }

  fn serialize_str(self, v: &str) -> Result<()> {
    let bytes = v.as_bytes();
    if bytes.len() == 1 {
      return self.serialize_char(bytes[0] as char);
    }

    let len = bytes.len();
    self.serialize_len(len, 0x08, 0x09, 0x0a)
      .ok_or(Error::StringExceedsMaxSize)?;
    self.output.extend(bytes);

    Ok(())
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<()> {
    use ser::SerializeSeq;
    let mut seq = self.serialize_seq(Some(v.len()))?;
    for byte in v {
      seq.serialize_element(byte)?;
    }
    seq.end()
  }

  fn serialize_none(self) -> Result<()> {
    self.serialize_unit()
  }

  fn serialize_some<T>(self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(self)
  }

  fn serialize_unit(self) -> Result<()> {
    self.output.push(0x00);
    Ok(())
  }

  fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
    self.serialize_unit()
  }

  fn serialize_unit_variant(
    self,
    _name: &'static str,
    variant_index: u32,
    _variant: &'static str,
  ) -> Result<()> {
    self.serialize_u32(variant_index)
  }

  fn serialize_newtype_struct<T>(
    self,
    _name: &'static str,
    value: &T,
  ) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(self)
  }

  // ultra lossy, but whatever
  fn serialize_newtype_variant<T>(
    self,
    _name: &'static str,
    _variant_index: u32,
    _variant: &'static str,
    value: &T,
  ) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut *self)
  }

  fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
    let len = len.ok_or(Error::LengthRequired)?;
    self.serialize_len(len, 0x0b, 0x0c, 0x0d)
      .ok_or(Error::SequenceExceedsMaxSize)?;
    Ok(self)
  }

  fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
    self.serialize_seq(Some(len))
  }

  fn serialize_tuple_struct(
    self,
    _name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleStruct> {
    self.serialize_seq(Some(len))
  }
  
  // still ultra lossy, still whatever
  fn serialize_tuple_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    _variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleVariant> {
    self.serialize_seq(Some(len))
  }

  fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
    let len = len.ok_or(Error::LengthRequired)?;
    self.serialize_len(len, 0x0e, 0x0f, 0x10)
      .ok_or(Error::MapExceedsMaxSize)?;
    Ok(self)
  }

  fn serialize_struct(
    self,
    _name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStruct> {
    self.serialize_map(Some(len))
  }
  
  // once again, ultra lossy, don't care
  fn serialize_struct_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    _variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStructVariant> {
    self.serialize_map(Some(len))
  }
}

impl ser::SerializeSeq for &mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_element<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl ser::SerializeTuple for &mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_element<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl ser::SerializeTupleStruct for &mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl ser::SerializeTupleVariant for &mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl ser::SerializeMap for &mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_key<T>(&mut self, key: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    key.serialize(&mut **self)
  }

  fn serialize_value<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl ser::SerializeStruct for &mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    key.serialize(&mut **self)?;
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}

impl ser::SerializeStructVariant for &mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    key.serialize(&mut **self)?;
    value.serialize(&mut **self)
  }

  fn end(self) -> Result<()> {
    Ok(())
  }
}


pub struct Deserializer {
  input: VecDeque<u8>,
}

impl Deserializer {
  pub fn from_bytes(bytes: &[u8]) -> Self {
    let mut vec = VecDeque::new();
    vec.extend(bytes.iter().copied());
    Deserializer { input: vec }
  }
}

pub fn from_bytes<'de, T>(bytes: &[u8]) -> Result<T>
where
  T: Deserialize<'de>,
{
  let mut deserializer = Deserializer::from_bytes(bytes);
  let t = T::deserialize(&mut deserializer)?;
  if deserializer.input.is_empty() {
    Ok(t)
  } else {
    Err(Error::TrailingCharacters)
  }
}

impl Deserializer {
  // look at the first character in the input without consuming it
  fn peek_byte(&mut self) -> Result<&u8> {
    self.input.front().ok_or(Error::Eof)
  }

  // consume the first character in the input
  fn next(&mut self) -> Result<u8> {
    self.input.pop_front().ok_or(Error::Eof)
  }

  // consume the next n characters
  fn take(&mut self, n: usize) -> Result<Vec<u8>> {
    Ok(self.input.drain(0..n).collect())
  }

  fn take_len(&mut self, base: u8) -> Result<usize> {
    Ok(match base {
      0 => u8::from_be_bytes([self.next()?]) as usize,
      1 => u16::from_be_bytes([
        self.next()?, self.next()?
      ]) as usize,
      2 => u32::from_be_bytes([
        self.next()?, self.next()?,
        self.next()?, self.next()?,
      ]) as usize,
      n => return Err(Error::UnknownType(n))
    })
  }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer {
  type Error = Error;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    match self.peek_byte()? {
      0x00 => self.deserialize_unit(visitor),
      0x01 => self.deserialize_i8(visitor),
      0x02 => self.deserialize_i16(visitor),
      0x03 => self.deserialize_i32(visitor),
      0x04 => self.deserialize_f64(visitor),
      0x05 | 0x06 => self.deserialize_bool(visitor),
      0x07 => self.deserialize_char(visitor),
      0x08 | 0x09 | 0x0a => self.deserialize_string(visitor),
      0x0b | 0x0c | 0x0d => self.deserialize_seq(visitor),
      0x0e | 0x0f | 0x10 => self.deserialize_map(visitor),
      n => Err(Error::UnknownType(*n)),
    }
  }

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_unit()
  }

  // The `parse_signed` function is generic over the integer type `T` so here
  // it is invoked with `T=i8`. The next 8 methods are similar.
  fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.next()?; // discard type
    visitor.visit_i8(i8::from_be_bytes([self.next()?]))
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.next()?; // discard type
    visitor.visit_i16(i16::from_be_bytes([
      self.next()?, self.next()?,
    ]))
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.next()?; // discard type
    visitor.visit_i32(i32::from_be_bytes([
      self.next()?, self.next()?,
      self.next()?, self.next()?,
    ]))
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.next()?; // discard type
    visitor.visit_f64(f64::from_be_bytes([
      self.next()?, self.next()?,
      self.next()?, self.next()?,
      self.next()?, self.next()?,
      self.next()?, self.next()?,
    ]))
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_bool(self.next()? == 0x05)
  }

  fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.next()?; // discard type
    visitor.visit_char(u8::from_be_bytes([self.next()?]) as char)
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let base = self.next()? - 0x08;
    let len = self.take_len(base)?;

    let bytes = self.take(len)?;
    let str = String::from_utf8(bytes)?;

    visitor.visit_string(str)
  }

  fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>
  { 
    let base = self.next()? - 0x0b;
    let len = self.take_len(base)?;

    visitor.visit_seq(Table::new(self, len))
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>
  {
    let base = self.next()? - 0x0e;
    let len = self.take_len(base)?;

    visitor.visit_map(Table::new(self, len))
  }

  forward_to_deserialize_any! {
    i64 i128 u8 u16 u32 u64 u128 f32
    str bytes byte_buf option unit_struct
    newtype_struct tuple tuple_struct struct enum
    identifier ignored_any
  }
}

struct Table<'a> {
  de: &'a mut Deserializer,
  len: usize,
}

impl<'a> Table<'a> {
  fn new(de: &'a mut Deserializer, len: usize) -> Self {
    Table { de, len }
  }
}

impl<'de, 'a> SeqAccess<'de> for Table<'a> {
  type Error = Error;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
  where
    T: DeserializeSeed<'de>,
  {
    if self.len == 0 {
      return Ok(None);
    }
    self.len -= 1;
    seed.deserialize(&mut *self.de).map(Some)
  }
}

impl<'de, 'a> MapAccess<'de> for Table<'a> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
  where
    K: DeserializeSeed<'de>,
  {
    if self.len == 0 {
      return Ok(None)
    }
    seed.deserialize(&mut *self.de).map(Some)
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
  where
    V: DeserializeSeed<'de>,
  {
    self.len -= 1;
    seed.deserialize(&mut *self.de)
  }
}

impl Serialize for Message {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: ser::Serializer,
  {
    match self {
      Message::Nil => serializer.serialize_unit(),
      Message::Int(n) => serializer.serialize_i32(*n),
      Message::Float(f) => serializer.serialize_f64(*f),
      Message::Boolean(b) => serializer.serialize_bool(*b),
      Message::String(s) => serializer.serialize_str(s),
      Message::Array(a) => a.serialize(serializer),
      Message::Map(m) => {
        use serde::ser::SerializeMap;
        let m = &m.inner;
        let mut map = serializer.serialize_map(Some(m.len()))?;
        for (k, v) in m {
          map.serialize_entry(&k, &v)?;
        }
        map.end()
      }
    }
  }
}

impl<'de> Deserialize<'de> for Message {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: de::Deserializer<'de>,
  {
    struct ValueVisitor;

    impl<'de> Visitor<'de> for ValueVisitor {
      type Value = Message;

      fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("any valid value")
      }
      
      fn visit_unit<E>(self) -> std::result::Result<Message, E> {
        Ok(Message::Nil)
      }
      fn visit_i8<E>(self, v: i8) -> std::result::Result<Message, E> {
        Ok(Message::Int(v.into()))
      }
      fn visit_i16<E>(self, v: i16) -> std::result::Result<Message, E> {
        Ok(Message::Int(v.into()))
      }
      fn visit_i32<E>(self, v: i32) -> std::result::Result<Message, E> {
        Ok(Message::Int(v))
      }
      fn visit_f64<E>(self, v: f64) -> std::result::Result<Message, E> {
        Ok(Message::Float(v))
      }
      fn visit_bool<E>(self, v: bool) -> std::result::Result<Message, E> {
        Ok(Message::Boolean(v))
      }
      fn visit_char<E>(self, v: char) -> std::result::Result<Message, E> {
        Ok(Message::String(v.to_string()))
      }
      fn visit_str<E>(self, v: &str) -> std::result::Result<Message, E> {
        Ok(Message::String(v.to_owned()))
      }

      fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Message, A::Error>
      where
        A: SeqAccess<'de>,
      {
        let mut vec = Vec::new();
        while let Some(elem) = seq.next_element()? {
          vec.push(elem);
        }
        Ok(Message::Array(vec))
      }

      fn visit_map<A>(self, mut map: A) -> std::result::Result<Message, A::Error>
      where
        A: MapAccess<'de>,
      {
        let mut vec = Vec::new();
        while let Some((k, v)) = map.next_entry()? {
          vec.push((k, v));
        }
        Ok(Message::Map(Map::from(vec)))
      }
    }

    deserializer.deserialize_any(ValueVisitor)
  }
}