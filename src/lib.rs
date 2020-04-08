pub mod bytes;
pub mod compatibility;
pub mod events;
pub mod telnet;

#[cfg(test)]
mod tests;

use compatibility::*;
use events::*;

pub struct Parser {
    options: CompatibilityTable,
    buffer: Vec<u8>,
    events: Vec<TelnetEvent>,
}

impl Iterator for Parser {
    type Item = TelnetEvent;
    fn next(&mut self) -> Option<TelnetEvent> {
        let item = &self.get_event();
        match item {
            Some(ev) => Some((*ev).clone()),
            None => None,
        }
    }
}

impl Default for Parser {
    fn default() -> Parser {
        Parser {
            options: CompatibilityTable::new(),
            buffer: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn receive(&mut self, data: &[u8]) {
        self.buffer = bytes::concat(&self.buffer, data);
        self.process();
    }
    pub fn escape_iac(data: Vec<u8>) -> Vec<u8> {
        let mut t: Vec<u8> = Vec::new();
        for val in data {
            if val == 255 {
                t.push(val);
            }
            t.push(val);
        }
        t
    }
    pub fn unescape_iac(data: Vec<u8>) -> Vec<u8> {
        let mut t: Vec<u8> = Vec::new();
        for (index, val) in data.iter().enumerate() {
            if *val == 255 && data[index + 1] == 255 {
                continue;
            }
            t.push(*val);
        }
        t
    }
    fn get_event(&mut self) -> Option<TelnetEvent> {
        let item = self.events.get(0);
        let res: Option<TelnetEvent>;
        match item {
            Some(ev) => res = Some(ev.clone()),
            None => res = None,
        };
        if self.events.len() > 1 {
            self.events = Vec::from(&self.events[1..]);
        } else {
            self.events = Vec::new();
        }
        res
    }
    fn push_event(&mut self, event: TelnetEvent) {
        self.events.push(event);
    }
    pub fn send(&mut self, data: &[u8]) {
        self.push_event(TelnetEvent::Send(DataEvent {
            size: data.len(),
            buffer: Vec::from(data),
        }));
    }
    pub fn send_text(&mut self, text: &str) {
        self.send(&bytes::concat(
            &Parser::escape_iac(format!("{}\r\n", text).into_bytes()),
            &[255, 249],
        ));
    }
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
                            self.events.push(TelnetEvent::IAC(buffer[1]));
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
                                        self.options.set_option(buffer[2], opt);
                                        self.send(&[255, 253, buffer[2]]);
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
                                        self.options.set_option(buffer[2], opt);
                                        self.send(&[255, 251, buffer[2]]);
                                    } else if !opt.local {
                                        self.send(&[255, 254, buffer[2]]);
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
                                _ => self.events.push(TelnetEvent::Negotiation(NegotiationEvent {
                                    command: buffer[1],
                                    option: buffer[2],
                                })),
                            }
                        }
                    }
                    _ => {
                        // Must be subnegotiation?
                        let len: usize = buffer.len();
                        if buffer[len - 2] == 255 && buffer[len - 1] == 240 {
                            // Valid ending
                            let opt = self.options.get_option(buffer[2]);
                            if opt.local {
                                self.push_event(TelnetEvent::Subnegotiation(SubnegotiationEvent {
                                    option: buffer[2],
                                    buffer: Vec::from(&buffer[3..len - 2]),
                                }));
                            }
                        } else {
                            // Missing the rest
                            self.buffer = bytes::concat(&self.buffer, &buffer);
                        }
                    }
                }
            } else {
                // Not an iac sequence, it's data!
                self.events.push(TelnetEvent::Data(DataEvent {
                    size: buffer.len(),
                    buffer: buffer.clone(),
                }));
            }
        }
    }
}
