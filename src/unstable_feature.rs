use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub(crate) enum UnstableFeature {
  FormatSubcommand,
  ScriptAttribute,
}

impl Display for UnstableFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::FormatSubcommand => write!(f, "The `--fmt` command is currently unstable."),
      Self::ScriptAttribute => write!(f, "The `[script]` attribute is currently unstable."),
    }
  }
}
