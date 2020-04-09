use super::*;

struct TelEvent;

impl TelnetEvents for TelEvent {
  fn on_iac(&self, command: u8) {
    println!("IAC: {}", command);
  }
  fn on_data(&self, size: usize, buffer: Vec<u8>) {
    println!(
      "Data: {} byte(s) | {}",
      size,
      String::from_utf8(buffer).unwrap()
    );
  }
  fn on_send(&self, size: usize, buffer: Vec<u8>) {
    println!("Send: {} byte(s) | {:?}", size, buffer);
  }
  fn on_negotiation(&self, command: u8, option: u8) {
    println!("Negotiate: {} {}", command, option);
  }
  fn on_subnegotiation(&self, option: u8, size: usize, buffer: Vec<u8>) {
    match String::from_utf8(buffer.clone()) {
      Ok(text) => {
        println!("Subnegotiation: {} - {} byte(s) | {}", option, size, text);
      }
      Err(_) => {
        println!(
          "Subnegotiation: {} - {} byte(s) | {:?}",
          option, size, buffer
        );
      }
    }
  }
}

/// Test the parser and its general functionality.
#[test]
fn test_parser() {
  let mut instance: Parser = Parser::new();
  instance.add_hooks(TelEvent);
  instance.options.support_local(201);
  instance._will(201);
  instance.receive(&bytes::concat(b"Hello, rust!", &[255, 249]));
  instance.receive(&[255, 253, 201]);
  instance.receive(&[255, 250, 201]);
  instance.receive(b"Core.Hello {}");
  instance.receive(&[255, 240]);
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
