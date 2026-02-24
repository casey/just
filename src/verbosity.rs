#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
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
    self >= Self::Loquacious
  }

  pub(crate) fn grandiloquent(self) -> bool {
    self >= Self::Grandiloquent
  }

  pub(crate) const fn default() -> Self {
    Self::Taciturn
  }
}

impl Default for Verbosity {
  fn default() -> Self {
    Self::default()
  }
}
