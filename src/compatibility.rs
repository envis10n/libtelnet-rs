/// An expansion of a bitmask contained in `CompatibilityTable`.
#[derive(Clone, Copy)]
pub struct CompatibilityEntry {
  /// Whether we support this option from us -> them.
  pub local: bool,
  /// Whether we support this option from them -> us.
  pub remote: bool,
  /// Whether this option is locally enabled.
  pub local_state: bool,
  /// Whether this option is remotely enabled.
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
  /// Creates a u8 bitmask from this entry.
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
  /// Expands a u8 bitmask into a CompatibilityEntry.
  pub fn from(value: u8) -> Self {
    Self {
      local: value & CompatibilityTable::ENABLED_LOCAL == CompatibilityTable::ENABLED_LOCAL,
      remote: value & CompatibilityTable::ENABLED_REMOTE == CompatibilityTable::ENABLED_REMOTE,
      local_state: value & CompatibilityTable::LOCAL_STATE == CompatibilityTable::LOCAL_STATE,
      remote_state: value & CompatibilityTable::REMOTE_STATE == CompatibilityTable::REMOTE_STATE,
    }
  }
}

/// A table of options that are supported locally or remotely, and their current state.
pub struct CompatibilityTable {
  options: [u8; 256],
}

impl Default for CompatibilityTable {
  fn default() -> Self {
    Self { options: [0; 256] }
  }
}

impl CompatibilityTable {
  /// Option is locally supported.
  pub const ENABLED_LOCAL: u8 = 1;
  /// Option is remotely supported.
  pub const ENABLED_REMOTE: u8 = 1 << 1;
  /// Option is currently enabled locally.
  pub const LOCAL_STATE: u8 = 1 << 2;
  /// Option is currently enabled remotely.
  pub const REMOTE_STATE: u8 = 1 << 3;
  pub fn new() -> Self {
    Self::default()
  }
  /// Create a table with some option values set.
  ///
  /// # Arguments
  ///
  /// `values` - A slice of `(u8, u8)` tuples. The first value is the option code, and the second is the bitmask value for that option.
  ///
  /// # Notes
  ///
  /// An option bitmask can be generated using the `CompatibilityEntry` struct, using `entry.into_u8()`.
  pub fn from_options(values: &[(u8, u8)]) -> Self {
    let mut options: [u8; 256] = [0; 256];
    for (opt, val) in values {
      options[*opt as usize] = *val;
    }
    Self { options }
  }
  /// Enable local support for an option.
  pub fn support_local(&mut self, option: u8) {
    let mut opt = CompatibilityEntry::from(self.options[option as usize]);
    opt.local = true;
    self.set_option(option, opt);
  }
  /// Enable remote support for an option.
  pub fn support_remote(&mut self, option: u8) {
    let mut opt = CompatibilityEntry::from(self.options[option as usize]);
    opt.remote = true;
    self.set_option(option, opt);
  }
  /// Enable both remote and local support for an option.
  pub fn support(&mut self, option: u8) {
    let mut opt = CompatibilityEntry::from(self.options[option as usize]);
    opt.local = true;
    opt.remote = true;
    self.set_option(option, opt);
  }
  /// Retrieve a `CompatbilityEntry` generated from the current state of the option value.
  pub fn get_option(&self, option: u8) -> CompatibilityEntry {
    CompatibilityEntry::from(self.options[option as usize])
  }
  /// Set an option value by getting the bitmask from a `CompatibilityEntry`.
  pub fn set_option(&mut self, option: u8, entry: CompatibilityEntry) {
    self.options[option as usize] = entry.clone().into_u8();
  }
}
