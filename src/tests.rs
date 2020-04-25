use super::*;


/// Test the parser and its general functionality.

#[derive(PartialEq, Debug)]
enum Event {
    IAC,
    NEGOTIATION,
    SUBNEGOTIATION,
    RECV,
    SEND
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
            self.events.iter().zip(other.events.iter()).all(|(val1, val2)| { val1 == val2 })
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
                println!("Receive: {:?}", buffer);
                events.push(Event::RECV);
            }
            events::TelnetEvents::DataSend(buffer) => {
                println!("Send: {:?}", buffer);
                events.push(Event::SEND);
            }
        };
    }
    events
}

#[test]
fn test_parser() {
  let mut instance: Parser = Parser::new();
  instance.options.support_local(201);
  if let Some(ev) = instance._will(201) {
    assert_eq!(handle_events(vec![ev]), events![Event::SEND]);
  }
  assert_eq!(handle_events(instance.receive(&[b"Hello, rust!", &[255, 249][..]].concat())), events![Event::RECV, Event::IAC]);
  assert_eq!(handle_events(instance.receive(&[255, 253, 201])), events![]);
  assert_eq!(handle_events(
    instance.receive(&events::TelnetSubnegotiation::new(201, b"Core.Hello {}").into_bytes()),
  ), events![Event::SUBNEGOTIATION]);
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
