use crate::telnet::{op_command as cmd, op_option as opt};

use super::*;

/// Test the parser and its general functionality.

#[derive(PartialEq, Debug)]
enum Event {
  IAC,
  NEGOTIATION,
  SUBNEGOTIATION,
  RECV,
  SEND,
  DECOM,
}

macro_rules! events {
    ( $( $x:expr ),* ) => {
        {
            #[allow(unused_mut)]
            let mut temp_ce = CapturedEvents::default();
            $(
                temp_ce.push($x);
            )*
            temp_ce
        }
    };
}

#[derive(Default, Debug)]
struct CapturedEvents {
  events: Vec<Event>,
}

impl CapturedEvents {
  fn push(&mut self, event: Event) {
    self.events.push(event);
  }
}

impl PartialEq for CapturedEvents {
  fn eq(&self, other: &Self) -> bool {
    if self.events.len() == other.events.len() {
      self
        .events
        .iter()
        .zip(other.events.iter())
        .all(|(val1, val2)| val1 == val2)
    } else {
      false
    }
  }
}

fn handle_events(event_list: Vec<events::TelnetEvents>) -> CapturedEvents {
  let mut events = CapturedEvents::default();
  for event in event_list {
    match event {
      events::TelnetEvents::IAC(ev) => {
        println!("IAC: {}", ev.command);
        events.push(Event::IAC);
      }
      events::TelnetEvents::Negotiation(ev) => {
        println!("Negotiation: {} {}", ev.command, ev.option);
        events.push(Event::NEGOTIATION);
      }
      events::TelnetEvents::Subnegotiation(ev) => {
        println!("Subnegotiation: {} {:?}", ev.option, ev.buffer);
        events.push(Event::SUBNEGOTIATION);
      }
      events::TelnetEvents::DataReceive(buffer) => {
        println!(
          "Receive: {}",
          std::str::from_utf8(buffer.as_slice()).unwrap_or("Bad utf-8 bytes")
        );
        events.push(Event::RECV);
      }
      events::TelnetEvents::DataSend(buffer) => {
        println!("Send: {:?}", buffer);
        events.push(Event::SEND);
      }
      events::TelnetEvents::DecompressImmediate(buffer) => {
        println!("DECOMPRESS: {:?}", buffer);
        events.push(Event::DECOM);
      }
    };
  }
  events
}

#[test]
fn test_parser() {
  let mut instance: Parser = Parser::new();
  instance.options.support_local(201);
  instance.options.support_local(86);
  if let Some(ev) = instance._will(201) {
    assert_eq!(handle_events(vec![ev]), events![Event::SEND]);
  }
  if let Some(ev) = instance._will(86) {
    assert_eq!(handle_events(vec![ev]), events![Event::SEND]);
  }
  assert_eq!(
    handle_events(instance.receive(&[b"Hello, rust!", &[255, 249][..]].concat())),
    events![Event::RECV, Event::IAC]
  );
  assert_eq!(handle_events(instance.receive(&[255, 253, 201])), events![]);
  assert_eq!(
    handle_events(instance.receive(&[&[255, 253, 200][..], b"Some random data"].concat())),
    events![Event::SEND, Event::RECV]
  );
  assert_eq!(
    handle_events(
      instance.receive(&events::TelnetSubnegotiation::new(201, b"Core.Hello {}").into_bytes()),
    ),
    events![Event::SUBNEGOTIATION]
  );
  assert_eq!(
    handle_events(
      instance.receive(
        &[
          &events::TelnetSubnegotiation::new(201, b"Core.Hello {}").into_bytes()[..],
          b"Random text",
          &[255, 249][..]
        ]
        .concat()
      ),
    ),
    events![Event::SUBNEGOTIATION, Event::RECV, Event::IAC]
  );
  assert_eq!(
    handle_events(
      instance.receive(
        &[
          &events::TelnetSubnegotiation::new(86, b" ").into_bytes()[..],
          b"This is compressed data",
          &[255, 249][..]
        ]
        .concat()
      ),
    ),
    events![Event::SUBNEGOTIATION, Event::DECOM]
  );
  assert_eq!(
    handle_events(instance.receive(&[
      87, 104, 97, 116, 32, 105, 115, 32, 121, 111, 117, 114, 32, 112, 97, 115, 115, 119, 111, 114,
      100, 63, 32, 255, 239, 255, 251, 1
    ])),
    events![Event::RECV, Event::IAC, Event::SEND]
  );
}

#[test]
fn test_subneg_separate_receives() {
  let mut instance: Parser = Parser::with_capacity(10);
  instance.options.support_local(opt::GMCP);
  instance._will(opt::GMCP);
  let mut events = instance.receive(
    &[
      &[cmd::IAC, cmd::SB, opt::GMCP][..],
      b"Otion.Data { some: json, data: in, here: ! }",
    ]
    .concat(),
  );
  assert_eq!(handle_events(events), events![]);

  events = instance.receive(b"More.Data { some: json, data: in, here: ! }");
  assert_eq!(handle_events(events), events![]);

  events = instance.receive(
    &[
      &[cmd::IAC, cmd::SE][..],
      &[cmd::IAC, cmd::SB, opt::GMCP][..],
      b"Otion.Data { some: json, data: in, here: ! }",
    ]
    .concat(),
  );
  assert_eq!(handle_events(events), events![Event::SUBNEGOTIATION]);

  events = instance.receive(
    &[
      b"More.Data { some: json, data: in, here: ! }",
      &[cmd::IAC, cmd::SE][..],
    ]
    .concat(),
  );
  assert_eq!(handle_events(events), events![Event::SUBNEGOTIATION]);
}

#[test]
fn test_concat() {
  let a: &[u8] = &[255, 102, 50, 65, 20];
  let b: &[u8] = &[1, 2, 3];
  let c: &[u8] = &[4, 5, 6, 7, 8, 9, 0];
  let expected: Vec<u8> = vec![255, 102, 50, 65, 20, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
  let actual: Vec<u8> = [a, b, c].concat();
  assert_eq!(expected, actual);
}

/// Test escaping IAC bytes in a buffer.
#[test]
fn test_escape() {
  let a: Vec<u8> = vec![255, 250, 201, 255, 205, 202, 255, 240];
  let expected: Vec<u8> = vec![255, 255, 250, 201, 255, 255, 205, 202, 255, 255, 240];
  assert_eq!(expected, Parser::escape_iac(a))
}

/// Test unescaping IAC bytes in a buffer.
#[test]
fn test_unescape() {
  let a: Vec<u8> = vec![255, 255, 250, 201, 255, 255, 205, 202, 255, 255, 240];
  let expected: Vec<u8> = vec![255, 250, 201, 255, 205, 202, 255, 240];
  assert_eq!(expected, Parser::unescape_iac(a))
}

#[test]
fn sync_parser() {
  let mut parser = Parser::new();
  parser.init_channels();
  parser.options.support(telnet::op_option::GMCP);
  let inbound = parser.inbound_events();
  let outbound = parser.outbound_events();
  let temp_buf = vec![255, GA];
  parser.receive(&temp_buf);
  parser._do(telnet::op_option::GMCP);
  let ev_a = inbound.recv().unwrap();
  let ev_b = outbound.recv().unwrap();
  assert_eq!(ev_a, events::TelnetEvents::IAC(events::TelnetIAC::new(GA)));
  assert_eq!(ev_b, events::TelnetEvents::DataSend(vec![255, 253, 201]));
}
