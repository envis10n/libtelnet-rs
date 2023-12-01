use crate::Parser;
use alloc::vec::Vec;
use bytes::{BufMut, Bytes, BytesMut};

/// A struct representing a 2 byte IAC sequence.
#[derive(Clone, Copy, Debug)]
pub struct TelnetIAC {
  pub command: u8,
}

impl Into<Bytes> for TelnetIAC {
  fn into(self) -> Bytes {
    let mut buf = BytesMut::with_capacity(2);
    buf.put_u8(255);
    buf.put_u8(self.command);
    buf.freeze()
  }
}

impl Into<Vec<u8>> for TelnetIAC {
  fn into(self) -> Vec<u8> {
    let b: Bytes = self.into();
    b.to_vec()
  }
}

impl TelnetIAC {
  pub fn new(command: u8) -> Self {
    Self { command }
  }
  /// Consume the sequence struct and return the bytes.
  pub fn into_bytes(self) -> Vec<u8> {
    self.into()
  }
}

/// A struct representing a 3 byte IAC sequence.
#[derive(Clone, Copy, Debug)]
pub struct TelnetNegotiation {
  pub command: u8,
  pub option: u8,
}

impl Into<Bytes> for TelnetNegotiation {
  fn into(self) -> Bytes {
    let data = [self.command, self.option];
    let mut buf = BytesMut::with_capacity(3);
    buf.put_u8(255);
    buf.put(&data[..]);
    buf.freeze()
  }
}

impl Into<Vec<u8>> for TelnetNegotiation {
  fn into(self) -> Vec<u8> {
    let b: Bytes = self.into();
    b.to_vec()
  }
}

impl TelnetNegotiation {
  pub fn new(command: u8, option: u8) -> Self {
    Self { command, option }
  }
  /// Consume the sequence struct and return the bytes.
  pub fn into_bytes(self) -> Vec<u8> {
    self.into()
  }
}

/// A struct representing an arbitrary length IAC subnegotiation sequence.
#[derive(Clone, Debug)]
pub struct TelnetSubnegotiation {
  pub option: u8,
  pub buffer: Bytes,
}

impl Into<Bytes> for TelnetSubnegotiation {
  fn into(self) -> Bytes {
    let head: [u8; 3] = [255, 250, self.option];
    let parsed = &Parser::escape_iac(self.buffer)[..];
    let tail: [u8; 2] = [255, 240];
    let mut buf = BytesMut::with_capacity(head.len() + parsed.len() + tail.len());
    buf.put(&head[..]);
    buf.put(&parsed[..]);
    buf.put(&tail[..]);
    buf.freeze()
  }
}

impl Into<Vec<u8>> for TelnetSubnegotiation {
  fn into(self) -> Vec<u8> {
    let b: Bytes = self.into();
    b.to_vec()
  }
}

impl TelnetSubnegotiation {
  pub fn new(option: u8, buffer: Bytes) -> Self {
    Self { option, buffer }
  }
  /// Consume the sequence struct and return the bytes.
  pub fn into_bytes(self) -> Vec<u8> {
    self.into()
  }
}

/// An enum representing various telnet events.
#[derive(Clone, Debug)]
pub enum TelnetEvents {
  /// An IAC command sequence.
  IAC(TelnetIAC),
  /// An IAC negotiation sequence.
  Negotiation(TelnetNegotiation),
  /// An IAC subnegotiation sequence.
  Subnegotiation(TelnetSubnegotiation),
  /// Regular data received from the remote end.
  DataReceive(Bytes),
  /// Any data to be sent to the remote end.
  DataSend(Bytes),
  /// MCCP2/3 compatibility. MUST DECOMPRESS THIS DATA BEFORE PARSING
  DecompressImmediate(Bytes),
}

impl Into<Bytes> for TelnetEvents {
  fn into(self) -> Bytes {
    match self {
      TelnetEvents::IAC(iac) => iac.into(),
      TelnetEvents::Negotiation(neg) => neg.into(),
      TelnetEvents::Subnegotiation(sub) => sub.into(),
      TelnetEvents::DataReceive(data) => data,
      TelnetEvents::DataSend(data) => data,
      TelnetEvents::DecompressImmediate(data) => data,
    }
  }
}

impl TelnetEvents {
  /// Helper method to generate a TelnetEvents::DataSend.
  pub fn build_send(buffer: Bytes) -> Self {
    TelnetEvents::DataSend(buffer)
  }
  /// Helper method to generate a TelnetEvents::DataReceive.
  pub fn build_receive(buffer: Bytes) -> Self {
    TelnetEvents::DataReceive(buffer)
  }
  /// Helper method to generate a TelnetEvents::IAC.
  pub fn build_iac(command: u8) -> TelnetEvents {
    TelnetEvents::IAC(TelnetIAC::new(command))
  }
  /// Helper method to generate a TelnetEvents::Negotiation.
  pub fn build_negotiation(command: u8, option: u8) -> Self {
    TelnetEvents::Negotiation(TelnetNegotiation::new(command, option))
  }
  /// Helper method to generate a TelnetEvents::Subnegotiation.
  pub fn build_subnegotiation(option: u8, buffer: Bytes) -> Self {
    TelnetEvents::Subnegotiation(TelnetSubnegotiation::new(option, buffer))
  }
}
