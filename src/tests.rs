use super::*;

/// Test the parser and its general functionality.
#[test]
fn test_parser() {
  let mut instance: Parser = Parser::new();
  instance.options.support_local(201);
  instance._will(201);
  instance.receive(&bytes::concat(b"Hello, rust!", &[255, 249]));
  instance.receive(&[255, 253, 201]);
  instance.receive(&[255, 250, 201]);
  instance.receive(b"Core.Hello {}");
  instance.receive(&[255, 240]);
  for ev in instance {
    match ev {
      TelnetEvent::IAC(command) => println!("IAC: {:?}", command),
      TelnetEvent::Negotiation(nev) => println!("Negotiation: {} {}", nev.command, nev.option),
      TelnetEvent::Subnegotiation(sev) => {
        println!(
          "Subnegotiation: {} {:?}",
          sev.option,
          String::from_utf8(sev.buffer).expect("Error parsing subnegotiation data to UTF8 string")
        );
      }
      TelnetEvent::Data(dev) => println!("Data: {:?}", dev.buffer),
      TelnetEvent::Send(sdev) => println!("Send: {:?}", sdev.buffer),
    }
  }
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
