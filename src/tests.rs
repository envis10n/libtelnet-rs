use super::*;

/// Test the parser and its general functionality.

fn handle_events(event_list: Vec<events::TelnetEvents>) {
  for event in event_list {
    match event {
      events::TelnetEvents::IAC(ev) => {
        println!("IAC: {}", ev.command);
      }
      events::TelnetEvents::Negotiation(ev) => {
        println!("Negotiation: {} {}", ev.command, ev.option);
      }
      events::TelnetEvents::Subnegotiation(ev) => {
        println!("Subnegotiation: {} {:?}", ev.option, ev.buffer);
      }
      events::TelnetEvents::DataReceive(buffer) => {
        println!("Receive: {:?}", buffer);
      }
      events::TelnetEvents::DataSend(buffer) => {
        println!("Send: {:?}", buffer);
      }
    }
  }
}

#[test]
fn test_parser() {
  let mut instance: Parser = Parser::new();
  instance.options.support_local(201);
  if let Some(buffer) = instance._will(201) {
    handle_events(vec![events::TelnetEvents::build_send(buffer)]);
  }
  handle_events(instance.receive(&bytes::concat(b"Hello, rust!", &[255, 249])));
  handle_events(instance.receive(&[255, 253, 201]));
  handle_events(
    instance.receive(&events::TelnetSubnegotiation::new(201, b"Core.Hello {}").into_bytes()),
  );
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
