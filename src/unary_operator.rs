use super::*;

/// A unary expression operator.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Ord, PartialOrd)]
pub(crate) enum UnaryOperator {
  /// `!`
  Not,
  /// leading `/`
  Slash,
}

impl UnaryOperator {
  pub(crate) fn serialization(self) -> &'static str {
    match self {
      Self::Not => "not",
      Self::Slash => "/",
    }
  }
}

impl Display for UnaryOperator {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Not => write!(f, "!"),
      Self::Slash => write!(f, "/"),
    }
  }
}
