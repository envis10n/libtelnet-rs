pub mod op_command {
  pub const IAC: u8 = 255;
  /** Confirm  */
  pub const WILL: u8 = 251;
  /** Tell the other side that we refuse to use an option. */
  pub const WONT: u8 = 252;
  /** Request that the other side begin using an option. */
  pub const DO: u8 = 253;
  /**  */
  pub const DONT: u8 = 254;
  pub const NOP: u8 = 241;
  /** Subnegotiation used for sending out-of-band data. */
  pub const SB: u8 = 250;
  /** Marks the end of a subnegotiation sequence. */
  pub const SE: u8 = 240;
  pub const IS: u8 = 0;
  pub const SEND: u8 = 1;
  /** Go Ahead */
  pub const GA: u8 = 249;
}

pub mod op_option {
  pub const BINARY: u8 = 0;
  pub const ECHO: u8 = 1;
  pub const RCP: u8 = 2;
  pub const SGA: u8 = 3;
  pub const NAMS: u8 = 4;
  pub const STATUS: u8 = 5;
  pub const TM: u8 = 6;
  pub const RCTE: u8 = 7;
  pub const NAOL: u8 = 8;
  pub const NAOP: u8 = 9;
  pub const NAOCRD: u8 = 10;
  pub const NAOHTS: u8 = 11;
  pub const NAOHTD: u8 = 12;
  pub const NAOFFD: u8 = 13;
  pub const NAOVTS: u8 = 14;
  pub const NAOVTD: u8 = 15;
  pub const NAOLFD: u8 = 16;
  pub const XASCII: u8 = 17;
  pub const LOGOUT: u8 = 18;
  pub const BM: u8 = 19;
  pub const DET: u8 = 20;
  pub const SUPDUP: u8 = 21;
  pub const SUPDUPOUTPUT: u8 = 22;
  pub const SNDLOC: u8 = 23;
  pub const TTYPE: u8 = 24;
  pub const EOR: u8 = 25;
  pub const TUID: u8 = 26;
  pub const OUTMRK: u8 = 27;
  pub const TTYLOC: u8 = 28;
  pub const _3270REGIME: u8 = 29;
  pub const X3PAD: u8 = 30;
  pub const NAWS: u8 = 31;
  pub const TSPEED: u8 = 32;
  pub const LFLOW: u8 = 33;
  pub const LINEMODE: u8 = 34;
  pub const XDISPLOC: u8 = 35;
  pub const ENVIRON: u8 = 36;
  pub const AUTHENTICATION: u8 = 37;
  pub const ENCRYPT: u8 = 38;
  pub const NEWENVIRON: u8 = 39;
  pub const MSSP: u8 = 70;
  pub const ZMP: u8 = 93;
  pub const EXOPL: u8 = 255;
  pub const MCCP2: u8 = 86;
  pub const MCCP3: u8 = 87;
}
