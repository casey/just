use Verbosity::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Verbosity {
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

  pub(crate) fn loquacious(self) -> bool {
    match self {
      Taciturn => false,
      Loquacious | Grandiloquent => true,
    }
  }

  pub(crate) fn grandiloquent(self) -> bool {
    match self {
      Taciturn | Loquacious => false,
      Grandiloquent => true,
    }
  }
}
