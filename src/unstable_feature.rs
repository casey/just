use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub(crate) enum UnstableFeature {
  FormatSubcommand,
}

impl Display for UnstableFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::FormatSubcommand => write!(f, "The `--fmt` command is currently unstable."),
    }
  }
}
