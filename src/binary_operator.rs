use super::*;

/// A binary expression operator.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Ord, PartialOrd)]
pub(crate) enum BinaryOperator {
  /// `+`
  Concatenation,
  /// `==`
  Equality,
  /// `!=`
  Inequality,
  /// `/`
  Join,
  /// `&&`
  LogicalAnd,
  /// `||`
  LogicalOr,
  /// `=~`
  RegexMatch,
  /// `!~`
  RegexMismatch,
}

impl BinaryOperator {
  pub(crate) fn list_feature(self) -> Option<ListFeature> {
    match self {
      Self::Concatenation | Self::Join => None,
      Self::Equality | Self::Inequality | Self::RegexMatch | Self::RegexMismatch => {
        Some(ListFeature::ComparisonOperator)
      }
      Self::LogicalAnd | Self::LogicalOr => Some(ListFeature::LogicalOperator),
    }
  }

  pub(crate) fn serialization(self) -> &'static str {
    match self {
      Self::Concatenation => "concatenate",
      Self::Equality => "==",
      Self::Inequality => "!=",
      Self::Join => "join",
      Self::LogicalAnd => "and",
      Self::LogicalOr => "or",
      Self::RegexMatch => "=~",
      Self::RegexMismatch => "!~",
    }
  }
}

impl Display for BinaryOperator {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Concatenation => write!(f, "+"),
      Self::Equality => write!(f, "=="),
      Self::Inequality => write!(f, "!="),
      Self::Join => write!(f, "/"),
      Self::LogicalAnd => write!(f, "&&"),
      Self::LogicalOr => write!(f, "||"),
      Self::RegexMatch => write!(f, "=~"),
      Self::RegexMismatch => write!(f, "!~"),
    }
  }
}
