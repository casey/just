#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Verbosity {
  Quiet,
  Taciturn,
  Loquacious,
  Grandiloquent,
}

impl Verbosity {
  pub(crate) fn from_flag_occurrences(flag_occurrences: u8) -> Self {
    match flag_occurrences {
      0 => Self::Taciturn,
      1 => Self::Loquacious,
      _ => Self::Grandiloquent,
    }
  }

  pub(crate) fn quiet(self) -> bool {
    self == Self::Quiet
  }

  pub(crate) fn loud(self) -> bool {
    !self.quiet()
  }

  pub(crate) fn loquacious(self) -> bool {
    match self {
      Self::Quiet | Self::Taciturn => false,
      Self::Loquacious | Self::Grandiloquent => true,
    }
  }

  pub(crate) fn grandiloquent(self) -> bool {
    match self {
      Self::Quiet | Self::Taciturn | Self::Loquacious => false,
      Self::Grandiloquent => true,
    }
  }

  pub const fn default() -> Self {
    Self::Taciturn
  }
}

impl Default for Verbosity {
  fn default() -> Self {
    Self::default()
  }
}
