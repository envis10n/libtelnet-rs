#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub mod compatibility;
pub mod events;
pub mod telnet;

use alloc::{format, vec::Vec};
pub use bytes;

use crate::telnet::op_command::*;

use bytes::{BufMut, Bytes, BytesMut};
use compatibility::*;

pub enum EventType {
  None(Bytes),
  IAC(Bytes),
  SubNegotiation(Bytes, Option<Bytes>),
  Neg(Bytes),
}

#[macro_export]
/// Macro for calling `Bytes::copy_from_slice()`
macro_rules! vbytes {
  ($slice:expr) => {
    Bytes::copy_from_slice($slice)
  };
}

/// A telnet parser that handles the main parts of the protocol.
pub struct Parser {
  pub options: CompatibilityTable,
  buffer: BytesMut,
}

impl Default for Parser {
  fn default() -> Parser {
    Parser {
      options: CompatibilityTable::new(),
      buffer: BytesMut::with_capacity(128),
    }
  }
}

impl Parser {
  /// Create a default, empty Parser with an internal buffer capacity of 128 bytes.
  pub fn new() -> Self {
    Self::default()
  }
  /// Create an empty parser, setting the initial internal buffer capcity.
  pub fn with_capacity(size: usize) -> Self {
    Self {
      options: CompatibilityTable::new(),
      buffer: BytesMut::with_capacity(size),
    }
  }
  /// Create an parser, setting the initial internal buffer capacity and directly supplying a CompatibilityTable.
  pub fn with_support_and_capacity(size: usize, table: CompatibilityTable) -> Self {
    Self {
      options: table,
      buffer: BytesMut::with_capacity(size),
    }
  }
  /// Create a parser, directly supplying a CompatibilityTable.
  ///
  /// Uses the default initial buffer capacity of 128 bytes.
  pub fn with_support(table: CompatibilityTable) -> Self {
    Self {
      options: table,
      buffer: BytesMut::with_capacity(128),
    }
  }
  /// Receive bytes into the internal buffer.
  ///
  /// # Arguments
  ///
  /// * `data` - The bytes to be received. This should be sourced from the remote side of a connection.
  ///
  /// # Returns
  ///
  /// `Vec<events::TelnetEvents>` - Any events parsed from the internal buffer with the new bytes.
  ///
  pub fn receive(&mut self, data: &[u8]) -> Vec<events::TelnetEvents> {
    self.buffer.put(data);
    self.process()
  }
  /// Get whether the remote end supports and is using linemode.
  pub fn linemode_enabled(&mut self) -> bool {
    let opt = self.options.get_option(telnet::op_option::LINEMODE);
    opt.remote && opt.remote_state
  }
  /// Escape IAC bytes in data that is to be transmitted and treated as a non-IAC sequence.
  ///
  /// # Example
  /// `[255, 1, 6, 2]` -> `[255, 255, 1, 6, 2]`
  pub fn escape_iac<T>(data: T) -> Bytes
  where
    Bytes: From<T>,
  {
    let data = Bytes::from(data);
    let mut t = BytesMut::with_capacity(data.len());
    for byte in data.iter() {
      t.put_u8(*byte);
      if *byte == 255 {
        t.put_u8(255);
      }
    }
    t.freeze()
  }
  /// Reverse escaped IAC bytes for non-IAC sequences and data.
  ///
  /// # Example
  /// `[255, 255, 1, 6, 2]` -> `[255, 1, 6, 2]`
  pub fn unescape_iac<T>(data: T) -> Bytes
  where
    Bytes: From<T>,
  {
    let data = Bytes::from(data);
    let mut t = BytesMut::with_capacity(data.len());
    let mut last = 0u8;
    for val in data.iter() {
      if *val == 255 && last == 255 {
        continue;
      }
      last = *val;
      t.put_u8(*val);
    }
    t.freeze()
  }
  /// Negotiate an option.
  ///
  /// # Arguments
  ///
  /// `command` - A `u8` representing the telnet command code to be negotiated with. Example: WILL (251), WONT (252), DO (253), DONT (254)
  ///
  /// `option` - A `u8` representing the telnet option code that is being negotiated.
  ///
  /// # Returns
  ///
  /// `events::TelnetEvents::DataSend` - A DataSend event to be processed.
  ///
  /// # Usage
  ///
  /// This and other methods meant for sending data to the remote end will generate a `TelnetEvents::Send(DataEvent)` event.
  ///
  /// These Send events contain a buffer that should be sent directly to the remote end, as it will have already been encoded properly.
  pub fn negotiate(&mut self, command: u8, option: u8) -> events::TelnetEvents {
    events::TelnetEvents::build_send(events::TelnetNegotiation::new(command, option).into())
  }
  /// Indicate to the other side that you are able and wanting to utilize an option.
  ///
  /// # Arguments
  ///
  /// `option` - A `u8` representing the telnet option code that you want to enable locally.
  ///
  /// # Returns
  ///
  /// `Option<events::TelnetEvents::DataSend>` - The DataSend event to be processed, or None if not supported.
  ///
  /// # Notes
  ///
  /// This method will do nothing if the option is not "supported" locally via the `CompatibilityTable`.
  pub fn _will(&mut self, option: u8) -> Option<events::TelnetEvents> {
    let mut opt = self.options.get_option(option);
    if opt.local && !opt.local_state {
      opt.local_state = true;
      self.options.set_option(option, opt);
      Some(self.negotiate(251, option))
    } else {
      None
    }
  }
  /// Indicate to the other side that you are not wanting to utilize an option.
  ///
  /// # Arguments
  ///
  /// `option` - A `u8` representing the telnet option code that you want to disable locally.
  ///
  /// # Returns
  ///
  /// `Option<events::TelnetEvents::DataSend>` - A DataSend event to be processed, or None if the option is already disabled.
  ///
  pub fn _wont(&mut self, option: u8) -> Option<events::TelnetEvents> {
    let mut opt = self.options.get_option(option);
    if opt.local_state {
      opt.local_state = false;
      self.options.set_option(option, opt);
      Some(self.negotiate(252, option))
    } else {
      None
    }
  }
  /// Indicate to the other side that you would like them to utilize an option.
  ///
  /// # Arguments
  ///
  /// `option` - A `u8` representing the telnet option code that you want to enable remotely.
  ///
  /// # Returns
  ///
  /// `Option<events::TelnetEvents::DataSend>` - A DataSend event to be processed, or None if the option is not supported or already enabled.
  ///
  /// # Notes
  ///
  /// This method will do nothing if the option is not "supported" remotely via the `CompatibilityTable`.
  pub fn _do(&mut self, option: u8) -> Option<events::TelnetEvents> {
    let opt = self.options.get_option(option);
    if opt.remote && !opt.remote_state {
      Some(self.negotiate(253, option))
    } else {
      None
    }
  }
  /// Indicate to the other side that you would like them to stop utilizing an option.
  ///
  /// # Arguments
  ///
  /// `option` - A `u8` representing the telnet option code that you want to disable remotely.
  ///
  /// # Returns
  ///
  /// `Option<events::TelnetEvents::DataSend>` - A DataSend event to be processed, or None if the option is already disabled.
  ///
  pub fn _dont(&mut self, option: u8) -> Option<events::TelnetEvents> {
    let opt = self.options.get_option(option);
    if opt.remote_state {
      Some(self.negotiate(254, option))
    } else {
      None
    }
  }
  /// Send a subnegotiation for a locally supported option.
  ///
  /// # Arguments
  ///
  /// `option` - A `u8` representing the telnet option code for the negotiation.
  ///
  /// `data` - A `Bytes` containing the data to be sent in the subnegotiation. This data will have all IAC (255) byte values escaped.
  ///
  /// # Returns
  ///
  /// `Option<events::TelnetEvents::DataSend>` - A DataSend event to be processed, or None if the option is not supported or is currently disabled.
  ///
  /// # Notes
  ///
  /// This method will do nothing if the option is not "supported" locally via the `CompatibilityTable`.
  pub fn subnegotiation<T>(&mut self, option: u8, data: T) -> Option<events::TelnetEvents>
  where
    Bytes: From<T>,
  {
    let opt = self.options.get_option(option);
    if opt.local && opt.local_state {
      Some(events::TelnetEvents::build_send(
        events::TelnetSubnegotiation::new(option, Bytes::from(data)).into(),
      ))
    } else {
      None
    }
  }
  /// Send a subnegotiation for a locally supported option, using a string instead of raw byte values.
  ///
  /// # Arguments
  ///
  /// `option` - A `u8` representing the telnet option code for the negotiation.
  ///
  /// `text` - A `&str` representing the text to be sent in the subnegotation. This data will have all IAC (255) byte values escaped.
  ///
  /// # Returns
  ///
  /// `Option<events::TelnetEvents::DataSend>` - A DataSend event to be processed, or None if the option is not supported or is currently disabled.
  ///
  /// # Notes
  ///
  /// This method will do nothing if the option is not "supported" locally via the `CompatibilityTable`.
  pub fn subnegotiation_text(&mut self, option: u8, text: &str) -> Option<events::TelnetEvents> {
    self.subnegotiation(option, Bytes::copy_from_slice(&text.as_bytes()))
  }
  /// Directly send a string, with appended `\r\n`, to the remote end, along with an `IAC (255) GOAHEAD (249)` sequence.
  ///
  /// # Returns
  ///
  /// `events::TelnetEvents::DataSend` - A DataSend event to be processed.
  ///
  /// # Notes
  ///
  /// The string will have IAC (255) bytes escaped before being sent.
  pub fn send_text(&mut self, text: &str) -> events::TelnetEvents {
    events::TelnetEvents::build_send(Bytes::copy_from_slice(&Parser::escape_iac(
      format!("{}\r\n", text).into_bytes(),
    )))
  }

  /// Extract sub-buffers from the current buffer
  fn extract_event_data(&mut self) -> Vec<EventType> {
    enum State {
      Normal,
      IAC,
      Neg,
      Sub,
    }
    let mut iter_state = State::Normal;

    let mut events: Vec<EventType> = Vec::with_capacity(4);
    let iter = self.buffer.iter().enumerate();
    let mut cmd_begin: usize = 0;

    for (index, &val) in iter {
      match iter_state {
        State::Normal => {
          if val == IAC {
            if cmd_begin < index {
              events.push(EventType::None(vbytes!(&self.buffer[cmd_begin..index])));
            }
            cmd_begin = index;
            iter_state = State::IAC;
          }
        }
        State::IAC => {
          match val {
            IAC => iter_state = State::Normal, // Double IAC, ignore
            GA | EOR | NOP => {
              events.push(EventType::IAC(vbytes!(&self.buffer[cmd_begin..index + 1])));
              cmd_begin = index + 1;
              iter_state = State::Normal;
            }
            SB => iter_state = State::Sub,
            _ => iter_state = State::Neg, // WILL | WONT | DO | DONT | IS | SEND
          }
        }
        State::Neg => {
          events.push(EventType::Neg(vbytes!(&self.buffer[cmd_begin..index + 1])));
          cmd_begin = index + 1;
          iter_state = State::Normal;
        }
        State::Sub => {
          // Every sub negotiation should be of the form:
          //   IAC SB <option> <optional data> IAC SE
          // Meaning it must:
          //  * Be at least 5 bytes long.
          //  * Start with IAC SB
          //  * End with IAC SE
          let long_enough = index - cmd_begin >= 4;
          let has_prefix = self.buffer[cmd_begin] == IAC && self.buffer[cmd_begin + 1] == SB;
          let has_suffix = val == SE && self.buffer[index - 1] == IAC;
          if long_enough && has_prefix && has_suffix {
            let opt = &self.buffer[cmd_begin + 2];
            if *opt == telnet::op_option::MCCP2 || *opt == telnet::op_option::MCCP3 {
              // MCCP2/MCCP3 MUST DECOMPRESS DATA AFTER THIS!
              events.push(EventType::SubNegotiation(
                vbytes!(&self.buffer[cmd_begin..index + 1]),
                Some(vbytes!(&self.buffer[index + 1..])),
              ));
              cmd_begin = self.buffer.len();
              break;
            } else {
              events.push(EventType::SubNegotiation(
                vbytes!(&self.buffer[cmd_begin..index + 1]),
                None,
              ));
              cmd_begin = index + 1;
              iter_state = State::Normal;
            }
          }
        }
      }
    }
    if cmd_begin < self.buffer.len() {
      match iter_state {
        State::Sub => events.push(EventType::SubNegotiation(
          vbytes!(&self.buffer[cmd_begin..]),
          None,
        )),
        _ => events.push(EventType::None(vbytes!(&self.buffer[cmd_begin..]))),
      }
    }

    // Empty the buffer when we are done
    self.buffer.clear();
    events
  }

  /// The internal parser method that takes the current buffer and generates the corresponding events.
  fn process(&mut self) -> Vec<events::TelnetEvents> {
    let mut event_list: Vec<events::TelnetEvents> = Vec::with_capacity(2);
    for event in self.extract_event_data() {
      match event {
        EventType::None(buffer) | EventType::IAC(buffer) | EventType::Neg(buffer) => {
          if buffer.is_empty() {
            continue;
          }
          if buffer[0] == IAC {
            match buffer.len() {
              2 => {
                if buffer[1] != SE {
                  // IAC command
                  event_list.push(events::TelnetEvents::build_iac(buffer[1]));
                }
              }
              3 => {
                // Negotiation
                let mut opt = self.options.get_option(buffer[2]);
                let event = events::TelnetNegotiation::new(buffer[1], buffer[2]);
                match buffer[1] {
                  WILL => {
                    if opt.remote && !opt.remote_state {
                      opt.remote_state = true;
                      event_list.push(events::TelnetEvents::build_send(vbytes!(&[
                        IAC, DO, buffer[2]
                      ])));
                      self.options.set_option(buffer[2], opt);
                      event_list.push(events::TelnetEvents::Negotiation(event));
                    } else if !opt.remote {
                      event_list.push(events::TelnetEvents::build_send(vbytes!(&[
                        IAC, DONT, buffer[2]
                      ])));
                    }
                  }
                  WONT => {
                    if opt.remote_state {
                      opt.remote_state = false;
                      self.options.set_option(buffer[2], opt);
                      event_list.push(events::TelnetEvents::build_send(vbytes!(&[
                        IAC, DONT, buffer[2]
                      ])));
                    }
                    event_list.push(events::TelnetEvents::Negotiation(event));
                  }
                  DO => {
                    if opt.local && !opt.local_state {
                      opt.local_state = true;
                      opt.remote_state = true;
                      event_list.push(events::TelnetEvents::build_send(vbytes!(&[
                        IAC, WILL, buffer[2]
                      ])));
                      self.options.set_option(buffer[2], opt);
                      event_list.push(events::TelnetEvents::Negotiation(event));
                    } else if !opt.local {
                      event_list.push(events::TelnetEvents::build_send(vbytes!(&[
                        IAC, WONT, buffer[2]
                      ])));
                    }
                  }
                  DONT => {
                    if opt.local_state {
                      opt.local_state = false;
                      self.options.set_option(buffer[2], opt);
                      event_list.push(events::TelnetEvents::build_send(vbytes!(&[
                        IAC, WONT, buffer[2]
                      ])));
                    }
                    event_list.push(events::TelnetEvents::Negotiation(event));
                  }
                  _ => (),
                }
              }
              _ => (),
            }
          } else {
            // Not an iac sequence, it's data!
            event_list.push(events::TelnetEvents::build_receive(buffer));
          }
        }
        EventType::SubNegotiation(buffer, remaining) => {
          let len: usize = buffer.len();
          if buffer[len - 2] == IAC && buffer[len - 1] == SE {
            // Valid ending
            let opt = self.options.get_option(buffer[2]);
            if opt.local && opt.local_state && len - 2 >= 3 {
              let dbuffer = vbytes!(&buffer[3..len - 2]);
              event_list.push(events::TelnetEvents::build_subnegotiation(
                buffer[2], dbuffer,
              ));
              if let Some(rbuf) = remaining {
                event_list.push(events::TelnetEvents::DecompressImmediate(rbuf));
              }
            }
          } else {
            // Missing the rest
            self.buffer.put(&buffer[..]);
          }
        }
      }
    }
    event_list
  }
}
