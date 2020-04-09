/// An event representing a general IAC negotiation sequence.
#[derive(Clone)]
pub struct NegotiationEvent {
  pub command: u8,
  pub option: u8,
}

/// An event representing a subnegotiation IAC sequence.
#[derive(Clone)]
pub struct SubnegotiationEvent {
  pub option: u8,
  pub buffer: Vec<u8>,
}

/// An event representing general data.
#[derive(Clone)]
pub struct DataEvent {
  pub size: usize,
  pub buffer: Vec<u8>,
}

/// Telnet event types.
#[derive(Clone)]
pub enum TelnetEvent {
  /// A general IAC command sequence. Example: IAC (255) GA (249)
  IAC(u8),
  /// A general negotiation IAC sequence.
  Negotiation(NegotiationEvent),
  /// A subnegotiation IAC sequence.
  Subnegotiation(SubnegotiationEvent),
  /// Data received from remote.
  Data(DataEvent),
  /// Data to send to remote.
  Send(DataEvent),
}
