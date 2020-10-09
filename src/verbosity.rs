use Verbosity::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Verbosity {
  Quiet,
  Taciturn,
  Loquacious,
  Grandiloquent,
}

impl Verbosity {
  pub(crate) fn from_flag_occurrences(flag_occurences: u64) -> Self {
    match flag_occurences {
      0 => Taciturn,
      1 => Loquacious,
      _ => Grandiloquent,
    }
  }

  pub(crate) fn quiet(self) -> bool {
    matches!(self, Quiet)
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
}
