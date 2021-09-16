use crate::common::*;

/// A conditional expression operator.
#[derive(PartialEq, Debug, Copy, Clone)]
pub(crate) enum ConditionalOperator {
  /// `==`
  Equality,
  /// `!=`
  Inequality,
  /// `=~`
  Match,
}

impl Display for ConditionalOperator {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Equality => write!(f, "=="),
      Self::Inequality => write!(f, "!="),
      Self::Match => write!(f, "=~"),
    }
  }
}
