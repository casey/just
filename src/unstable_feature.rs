use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub(crate) enum UnstableFeature {
  LogicalOperators,
  UserDefinedFunction,
  WhichFunction,
}

impl Display for UnstableFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::LogicalOperators => write!(
        f,
        "the logical operators `&&` and `||` are currently unstable."
      ),
      Self::UserDefinedFunction => {
        write!(f, "user-defined functions are currently unstable.")
      }
      Self::WhichFunction => write!(f, "the `which()` function is currently unstable."),
    }
  }
}
