use Verbosity::*;

#[derive(Copy, Clone)]
pub enum Verbosity {
  Taciturn,
  Loquacious,
  Grandiloquent,
}

impl Verbosity {
  pub fn from_flag_occurrences(flag_occurences: u64) -> Verbosity {
    match flag_occurences {
      0 => Taciturn,
      1 => Loquacious,
      _ => Grandiloquent,
    }
  }

  pub fn loquacious(self) -> bool {
    match self {
      Taciturn => false,
      Loquacious => true,
      Grandiloquent => true,
    }
  }

  pub fn grandiloquent(self) -> bool {
    match self {
      Taciturn => false,
      Loquacious => false,
      Grandiloquent => true,
    }
  }
}
