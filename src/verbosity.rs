use Verbosity::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Verbosity {
  Quiet,
  Taciturn,
  Loquacious,
  Grandiloquent,
}

impl Verbosity {
  pub(crate) fn from_flag_occurrences(flag_occurrences: u64) -> Self {
    match flag_occurrences {
      0 => Taciturn,
      1 => Loquacious,
      _ => Grandiloquent,
    }
  }

  pub(crate) fn quiet(self) -> bool {
    matches!(self, Quiet)
  }

  pub(crate) fn loud(self) -> bool {
    !self.quiet()
  }

  pub(crate) fn loquacious(self) -> bool {
    matches!(self, Loquacious | Grandiloquent)
  }

  pub(crate) fn grandiloquent(self) -> bool {
    self == Grandiloquent
  }

  pub const fn default() -> Self {
    Taciturn
  }
}
