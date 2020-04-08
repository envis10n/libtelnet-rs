#[derive(Clone)]
pub struct NegotiationEvent {
  pub command: u8,
  pub option: u8,
}
#[derive(Clone)]
pub struct SubnegotiationEvent {
  pub option: u8,
  pub buffer: Vec<u8>,
}
#[derive(Clone)]
pub struct DataEvent {
  pub size: usize,
  pub buffer: Vec<u8>,
}

#[derive(Clone)]
pub enum TelnetEvent {
  IAC(u8),
  Negotiation(NegotiationEvent),
  Subnegotiation(SubnegotiationEvent),
  Data(DataEvent),
  Send(DataEvent),
}
