#[derive(Clone, Copy)]
pub struct CompatibilityEntry {
  pub local: bool,
  pub remote: bool,
  pub local_state: bool,
  pub remote_state: bool,
}

impl CompatibilityEntry {
  pub fn new(local: bool, remote: bool, local_state: bool, remote_state: bool) -> Self {
    Self {
      local,
      remote,
      local_state,
      remote_state,
    }
  }
  pub fn into_u8(self) -> u8 {
    let mut res: u8 = 0;
    if self.local {
      res |= CompatibilityTable::ENABLED_LOCAL;
    }
    if self.remote {
      res |= CompatibilityTable::ENABLED_REMOTE;
    }
    if self.local_state {
      res |= CompatibilityTable::LOCAL_STATE;
    }
    if self.remote_state {
      res |= CompatibilityTable::REMOTE_STATE;
    }
    res
  }
  pub fn from(value: u8) -> Self {
    Self {
      local: value & CompatibilityTable::ENABLED_LOCAL == CompatibilityTable::ENABLED_LOCAL,
      remote: value & CompatibilityTable::ENABLED_REMOTE == CompatibilityTable::ENABLED_REMOTE,
      local_state: value & CompatibilityTable::LOCAL_STATE == CompatibilityTable::LOCAL_STATE,
      remote_state: value & CompatibilityTable::REMOTE_STATE == CompatibilityTable::REMOTE_STATE,
    }
  }
}

pub struct CompatibilityTable {
  options: [u8; 256],
}

impl Default for CompatibilityTable {
  fn default() -> Self {
    Self { options: [0; 256] }
  }
}

impl CompatibilityTable {
  pub const ENABLED_LOCAL: u8 = 1;
  pub const ENABLED_REMOTE: u8 = 1 << 1;
  pub const LOCAL_STATE: u8 = 1 << 2;
  pub const REMOTE_STATE: u8 = 1 << 3;
  pub fn new() -> Self {
    Self::default()
  }
  pub fn get_option(&self, option: u8) -> CompatibilityEntry {
    CompatibilityEntry::from(self.options[option as usize])
  }
  pub fn set_option(&mut self, option: u8, entry: CompatibilityEntry) {
    self.options[option as usize] = entry.clone().into_u8();
  }
}
