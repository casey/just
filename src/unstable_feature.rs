use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub(crate) enum UnstableFeature {
  FormatSubcommand,
  GlobImports,
  LogicalOperators,
  WhichFunction,
}

impl Display for UnstableFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::FormatSubcommand => write!(f, "The `--fmt` command is currently unstable."),
      Self::GlobImports => write!(f, "Glob patterns in imports are currently unstable."),
      Self::LogicalOperators => write!(
        f,
        "The logical operators `&&` and `||` are currently unstable."
      ),
      Self::WhichFunction => write!(f, "The `which()` function is currently unstable."),
    }
  }
}
