use crate::bytes;
use crate::Parser;

/// A struct represting a 2 byte IAC sequence.
#[derive(Clone, Copy)]
pub struct TelnetIAC {
  pub command: u8,
}

impl TelnetIAC {
  pub fn new(command: u8) -> Self {
    Self { command }
  }
  /// Consume the sequence struct and return the bytes.
  pub fn into_bytes(self) -> Vec<u8> {
    vec![255, self.command]
  }
}

/// A struct representing a 3 byte IAC sequence.
#[derive(Clone, Copy)]
pub struct TelnetNegotiation {
  pub command: u8,
  pub option: u8,
}

impl TelnetNegotiation {
  pub fn new(command: u8, option: u8) -> Self {
    Self { command, option }
  }
  /// Consume the sequence struct and return the bytes.
  pub fn into_bytes(self) -> Vec<u8> {
    vec![255, self.command, self.option]
  }
}

/// A struct representing an arbitrary length IAC subnegotiation sequence.
#[derive(Clone)]
pub struct TelnetSubnegotiation {
  pub option: u8,
  pub buffer: Vec<u8>,
}

impl TelnetSubnegotiation {
  pub fn new(option: u8, buffer: &[u8]) -> Self {
    Self {
      option,
      buffer: Vec::from(buffer),
    }
  }
  /// Consume the sequence struct and return the bytes.
  pub fn into_bytes(self) -> Vec<u8> {
    let start: &[u8] = &[255, 250, self.option];
    let mid = bytes::concat(start, &Parser::escape_iac(self.buffer));
    bytes::concat(&mid, &[255, 240])
  }
}

/// An enum representing various telnet events.
#[derive(Clone)]
pub enum TelnetEvents {
  /// An IAC command sequence.
  IAC(TelnetIAC),
  /// An IAC negotiation sequence.
  Negotiation(TelnetNegotiation),
  /// An IAC subnegotiation sequence.
  Subnegotiation(TelnetSubnegotiation),
  /// Regular data received from the remote end.
  DataReceive(Vec<u8>),
  /// Any data to be sent to the remote end.
  DataSend(Vec<u8>),
}

impl TelnetEvents {
  /// Helper method to generate a TelnetEvents::DataSend.
  pub fn build_send(buffer: Vec<u8>) -> Self {
    TelnetEvents::DataSend(buffer)
  }
  /// Helper method to generate a TelnetEvents::DataReceive.
  pub fn build_receive(buffer: Vec<u8>) -> Self {
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
  pub fn build_subnegotiation(option: u8, buffer: Vec<u8>) -> Self {
    TelnetEvents::Subnegotiation(TelnetSubnegotiation::new(option, &buffer))
  }
}
