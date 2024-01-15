use Verbosity::*;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub(crate) enum Verbosity {
  #[default]
  Quiet,
  Taciturn,
  Loquacious,
  Grandiloquent,
}

impl Verbosity {
  pub(crate) fn from_flag_occurrences(flag_occurrences: u8) -> Self {
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
    match self {
      Quiet | Taciturn => false,
      Loquacious | Grandiloquent => true,
    }
  }

  pub(crate) fn grandiloquent(self) -> bool {
    match self {
      Quiet | Taciturn | Loquacious => false,
      Grandiloquent => true,
    }
  }

  pub const fn default() -> Self {
    Taciturn
  }
}
