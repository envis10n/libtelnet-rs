use bytes::Bytes;
use libtelnet_rs::telnet::{op_command as cmd, op_option as opt};
use libtelnet_rs::vbytes;

use libtelnet_rs::*;
use libtelnet_rs::compatibility::{CompatibilityEntry, CompatibilityTable};

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
          std::str::from_utf8(&buffer[..]).unwrap_or("Bad utf-8 bytes")
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
      instance.receive(
        &events::TelnetSubnegotiation::new(201, Bytes::copy_from_slice(b"Core.Hello {}"))
          .into_bytes()
      ),
    ),
    events![Event::SUBNEGOTIATION]
  );
  assert_eq!(
    handle_events(
      instance.receive(
        &[
          &events::TelnetSubnegotiation::new(201, Bytes::copy_from_slice(b"Core.Hello {}"))
            .into_bytes()[..],
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
          &events::TelnetSubnegotiation::new(86, Bytes::copy_from_slice(b" ")).into_bytes()[..],
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

// Test that receiving a subnegotiation with embedded UTF-8 content works correctly,
// even when the content includes a SE byte.
#[test]
fn test_subneg_utf8_content() {
    use crate::events::TelnetEvents;
    use cmd::{IAC, SB, SE};
    use opt::GMCP;

    // Create a parser that will support GMCP.
    let mut parser = Parser::new();
    parser.options.support_local(GMCP);
    parser._will(GMCP);

    // Construct a GMCP message containing a UTF-8 sequence that happens
    // to include SE (0xF0). This should be permitted as long as the SE isn't
    // preceeded by IAC (0xFF). For our test case we'll use the content
    // '👋' (0xF0, 0x9F, 0x91, 0x8B) - where the leading byte is SE.
    let prefix = &[IAC, SB, GMCP][..];
    let wave_emoji = &[0xF0, 0x9F, 0x91, 0x8B][..];
    let suffix = &[IAC, SE][..];
    let gmcp_msg = [prefix, wave_emoji, suffix].concat();

    // Receive the GMCP message with the parser. This should produce one event.
    let events = parser.receive(&gmcp_msg);
    assert_eq!(events.len(), 1, "only expected one event to be parsed");

    // The event should be a Subnegotiation for the GMCP option, with the correct in-tact
    // buffer contents.
    if let TelnetEvents::Subnegotiation(sub) = events.get(0).unwrap() {
        assert_eq!(sub.option, 201, "option should be GMCP");
        assert_eq!(
            sub.buffer, wave_emoji,
            "buffer should be equal to the wave emoji"
        );
    } else {
        panic!("missing expected DataReceive event");
    }
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
  let a = vec![255, 250, 201, 255, 205, 202, 255, 240];
  let expected = vbytes!(&[255, 255, 250, 201, 255, 255, 205, 202, 255, 255, 240]);
  assert_eq!(expected, Parser::escape_iac(a))
}

/// Test unescaping IAC bytes in a buffer.
#[test]
fn test_unescape() {
  let a = vec![255, 255, 250, 201, 255, 255, 205, 202, 255, 255, 240];
  let expected = vbytes!(&[255, 250, 201, 255, 205, 202, 255, 240]);
  assert_eq!(expected, Parser::unescape_iac(a))
}

#[test]
fn test_bad_subneg_dbuffer() {
  // Configure opt 0xFF (IAC) as local supported, and local state enabled.
  let entry = CompatibilityEntry::new(true, false, true, false);
  let opts = CompatibilityTable::from_options(&[(
    cmd::IAC,
    entry.into_u8(),
  )]);
  // Receive a malformed subnegotiation - this should not panic.
  Parser::with_support(opts).receive(&[
    cmd::IAC,
    cmd::SB,
    cmd::IAC,
    cmd::SE,
  ]);
}
