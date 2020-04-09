pub mod bytes;
pub mod compatibility;
pub mod telnet;

#[cfg(test)]
mod tests;

use compatibility::*;

/// A trait for event handlers of Telnet events.
///
/// # Example
///
/// ```
/// struct TelEvent;
///
/// impl TelnetEvents for TelEvent {
///   fn on_iac(&self, command: u8) {
///     println!("IAC: {}", command);
///   }
///   fn on_data(&self, size: usize, buffer: Vec<u8>) {
///     println!(
///       "Data: {} byte(s) | {}",
///       size,
///       String::from_utf8(buffer).unwrap()
///     );
///   }
///   fn on_send(&self, size: usize, buffer: Vec<u8>) {
///     println!("Send: {} byte(s) | {:?}", size, buffer);
///   }
///   fn on_negotiation(&self, command: u8, option: u8) {
///     println!("Negotiate: {} {}", command, option);
///   }
///   fn on_subnegotiation(&self, option: u8, size: usize, buffer: Vec<u8>) {
///     match String::from_utf8(buffer.clone()) {
///       Ok(text) => {
///         println!("Subnegotiation: {} - {} byte(s) | {}", option, size, text);
///       }
///       Err(_) => {
///         println!(
///           "Subnegotiation: {} - {} byte(s) | {:?}",
///           option, size, buffer
///         );
///       }
///     }
///   }
/// }
/// ```
///
#[allow(unused_variables)]
pub trait TelnetEvents {
    fn on_iac(&mut self, command: u8) {}
    fn on_negotiation(&mut self, command: u8, option: u8) {}
    fn on_subnegotiation(&mut self, option: u8, size: usize, buffer: Vec<u8>) {}
    fn on_data(&mut self, size: usize, buffer: Vec<u8>) {}
    fn on_send(&mut self, size: usize, buffer: Vec<u8>) {}
}

/// A telnet parser that handles the main parts of the protocol.
pub struct Parser {
    pub options: CompatibilityTable,
    buffer: Vec<u8>,
    hooks: Vec<Box<dyn TelnetEvents>>,
}

impl Default for Parser {
    fn default() -> Parser {
        Parser {
            options: CompatibilityTable::new(),
            buffer: Vec::new(),
            hooks: Vec::new(),
        }
    }
}

impl Parser {
    /// Create a default, empty Parser.
    pub fn new() -> Self {
        Self::default()
    }
    /// Create a parser, directly supplying a CompatibilityTable.
    pub fn with_support(table: CompatibilityTable) -> Self {
        Self {
            options: table,
            buffer: Vec::new(),
            hooks: Vec::new(),
        }
    }
    /// Add a struct that implements the event hooks called when processing.
    ///
    /// # Arguments
    ///
    /// `hook` - A struct implementing the TelnetEvents trait.
    ///
    pub fn add_hooks<E: TelnetEvents + 'static>(&mut self, hook: E) {
        self.hooks.push(Box::new(hook));
    }
    /// Receive bytes into the internal buffer.
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to be received. This should be sourced from the remote side of a connection.
    ///
    pub fn receive(&mut self, data: &[u8]) {
        self.buffer = bytes::concat(&self.buffer, data);
        self.process();
    }
    /// Escape IAC bytes in data that is to be transmitted and treated as a non-IAC sequence.
    ///
    /// # Example
    /// `[255, 1, 6, 2]` -> `[255, 255, 1, 6, 2]`
    pub fn escape_iac(data: Vec<u8>) -> Vec<u8> {
        let mut t = data.clone();
        let mut c: usize = 0;
        for (i, byte) in data.iter().enumerate() {
            if *byte == 255 {
                t.insert(i + c, 255);
                c += 1;
            }
        }
        t
    }
    /// Reverse escaped IAC bytes for non-IAC sequences and data.
    ///
    /// # Example
    /// `[255, 255, 1, 6, 2]` -> `[255, 1, 6, 2]`
    pub fn unescape_iac(data: Vec<u8>) -> Vec<u8> {
        let mut t = data.clone();
        let mut c: usize = 0;
        for (index, val) in data.iter().enumerate() {
            if *val == 255 && data[index + 1] == 255 {
                t.remove(index - c);
                c += 1;
            }
        }
        t
    }
    /// Negotiate an option.
    ///
    /// # Arguments
    ///
    /// `command` - A `u8` representing the telnet command code to be negotiated with. Example: WILL (251), WONT (252), DO (253), DONT (254)
    ///
    /// `option` - A `u8` representing the telnet option code that is being negotiated.
    ///
    /// # Usage
    ///
    /// This and other methods meant for sending data to the remote end will generate a `TelnetEvents::Send(DataEvent)` event.
    ///
    /// These Send events contain a buffer that should be sent directly to the remote end, as it will have already been encoded properly.
    pub fn negotiate(&mut self, command: u8, option: u8) {
        self.send(&[255, command, option]);
    }
    /// Indicate to the other side that you are able and wanting to utilize an option.
    ///
    /// # Arguments
    ///
    /// `option` - A `u8` representing the telnet option code that you want to enable locally.
    ///
    /// # Notes
    ///
    /// This method will do nothing if the option is not "supported" locally via the `CompatibilityTable`.
    pub fn _will(&mut self, option: u8) {
        let mut opt = self.options.get_option(option);
        if opt.local && !opt.local_state {
            opt.local_state = true;
            self.negotiate(251, option);
            self.options.set_option(option, opt);
        }
    }
    /// Indicate to the other side that you are not wanting to utilize an option.
    ///
    /// # Arguments
    ///
    /// `option` - A `u8` representing the telnet option code that you want to disable locally.
    ///
    pub fn _wont(&mut self, option: u8) {
        let mut opt = self.options.get_option(option);
        if opt.local_state {
            opt.local_state = false;
            self.negotiate(252, option);
            self.options.set_option(option, opt);
        }
    }
    /// Indicate to the other side that you would like them to utilize an option.
    ///
    /// # Arguments
    ///
    /// `option` - A `u8` representing the telnet option code that you want to enable remotely.
    ///
    /// # Notes
    ///
    /// This method will do nothing if the option is not "supported" remotely via the `CompatibilityTable`.
    pub fn _do(&mut self, option: u8) {
        let opt = self.options.get_option(option);
        if opt.remote && !opt.remote_state {
            self.negotiate(253, option);
        }
    }
    /// Indicate to the other side that you would like them to stop utilizing an option.
    ///
    /// # Arguments
    ///
    /// `option` - A `u8` representing the telnet option code that you want to disable remotely.
    ///
    pub fn _dont(&mut self, option: u8) {
        let opt = self.options.get_option(option);
        if opt.remote_state {
            self.negotiate(254, option);
        }
    }
    /// Send a subnegotiation for a locally supported option.
    ///
    /// # Arguments
    ///
    /// `option` - A `u8` representing the telnet option code for the negotiation.
    ///
    /// `data` - A `Vec<u8>` containing the data to be sent in the subnegotiation. This data will have all IAC (255) byte values escaped.
    ///
    /// # Notes
    ///
    /// This method will do nothing if the option is not "supported" locally via the `CompatibilityTable`.
    pub fn subnegotiation(&mut self, option: u8, data: Vec<u8>) {
        let opt = self.options.get_option(option);
        if opt.local && opt.local_state {
            self.send(&bytes::concat(
                &[255, 250, option],
                &bytes::concat(&Parser::escape_iac(data), &[255, 240]),
            ));
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
    /// # Notes
    ///
    /// This method will do nothing if the option is not "supported" locally via the `CompatibilityTable`.
    pub fn subnegotiation_text(&mut self, option: u8, text: &str) {
        self.subnegotiation(option, String::from(text).into_bytes());
    }
    /// Directly send a buffer to the remote end.
    ///
    /// # Notes
    ///
    /// The buffer supplied here will NOT be escaped. It is recommended to avoid using this method in favor of the more specialized methods.
    pub fn send(&mut self, data: &[u8]) {
        for hook in &mut self.hooks {
            hook.on_send(data.len(), Vec::from(data));
        }
    }
    /// Directly send a string, with appended `\r\n`, to the remote end, along with an `IAC (255) GOAHEAD (249)` sequence.
    ///
    /// # Notes
    /// The string will have IAC (255) bytes escaped before being sent.
    pub fn send_text(&mut self, text: &str) {
        self.send(&bytes::concat(
            &Parser::escape_iac(format!("{}\r\n", text).into_bytes()),
            &[255, 249],
        ));
    }
    /// The internal parser method that takes the current buffer and generates the corresponding events.
    fn process(&mut self) {
        let mut t: Vec<Vec<u8>> = Vec::new();
        let iter = self.buffer.iter().enumerate();
        let mut offset_next: usize;
        let mut offset_last: usize = 0;
        for (index, &val) in iter {
            if val == 255 && self.buffer[index + 1] != 255 {
                offset_next = index;
                if offset_next != offset_last {
                    if self.buffer[offset_last + 1] == 250 && self.buffer[offset_next + 1] == 240 {
                        offset_next += 2;
                    }
                    t.push(Vec::from(&self.buffer[offset_last..offset_next]));
                    offset_last = offset_next;
                }
            }
        }
        if offset_last < self.buffer.len() {
            t.push(Vec::from(&self.buffer[offset_last..]));
        }
        self.buffer = Vec::new();
        for buffer in t {
            if buffer[0] == 255 {
                match buffer.len() {
                    2 => {
                        if buffer[1] != 240 {
                            // IAC command
                            for hook in &mut self.hooks {
                                hook.on_iac(buffer[1]);
                            }
                        }
                    }
                    3 => {
                        if buffer[1] == 250 {
                            // Subnegotiation but not complete yet.
                            self.buffer = bytes::concat(&self.buffer, &buffer);
                        } else {
                            // Negotiation
                            let mut opt = self.options.get_option(buffer[2]);
                            match buffer[1] {
                                251 => {
                                    // WILL
                                    if opt.remote && !opt.remote_state {
                                        opt.remote_state = true;
                                        self.send(&[255, 253, buffer[2]]);
                                        self.options.set_option(buffer[2], opt);
                                    } else if !opt.remote {
                                        self.send(&[255, 254, buffer[2]]);
                                    }
                                }
                                252 => {
                                    // WONT
                                    if opt.remote_state {
                                        opt.remote_state = false;
                                        self.options.set_option(buffer[2], opt);
                                        self.send(&[255, 254, buffer[2]]);
                                    }
                                }
                                253 => {
                                    // DO
                                    if opt.local && !opt.local_state {
                                        opt.local_state = true;
                                        opt.remote_state = true;
                                        self.send(&[255, 251, buffer[2]]);
                                        self.options.set_option(buffer[2], opt);
                                    } else if !opt.local {
                                        self.send(&[255, 252, buffer[2]]);
                                    }
                                }
                                254 => {
                                    // DONT
                                    if opt.local_state {
                                        opt.local_state = false;
                                        self.options.set_option(buffer[2], opt);
                                        self.send(&[255, 252, buffer[2]]);
                                    }
                                }
                                _ => (),
                            }
                            for hook in &mut self.hooks {
                                hook.on_negotiation(buffer[1], buffer[2]);
                            }
                        }
                    }
                    _ => {
                        // Must be subnegotiation?
                        let len: usize = buffer.len();
                        if buffer[len - 2] == 255 && buffer[len - 1] == 240 {
                            // Valid ending
                            let opt = self.options.get_option(buffer[2]);
                            if opt.local && opt.local_state {
                                let dbuffer = Vec::from(&buffer[3..len - 2]);
                                for hook in &mut self.hooks {
                                    hook.on_subnegotiation(
                                        buffer[2],
                                        dbuffer.len(),
                                        dbuffer.clone(),
                                    );
                                }
                            }
                        } else {
                            // Missing the rest
                            self.buffer = bytes::concat(&self.buffer, &buffer);
                        }
                    }
                }
            } else {
                // Not an iac sequence, it's data!
                for hook in &mut self.hooks {
                    hook.on_data(buffer.len(), buffer.clone())
                }
            }
        }
    }
}
