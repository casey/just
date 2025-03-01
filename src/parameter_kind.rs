use super::*;

/// Parameters can either be…
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ParameterKind {
  /// …variadic, accepting one or more arguments
  Plus,
  /// …singular, accepting a single argument
  Singular,
  /// …variadic, accepting zero or more arguments
  Star,
}

impl ParameterKind {
  pub(crate) fn prefix(self) -> Option<&'static str> {
    match self {
      Self::Singular => None,
      Self::Plus => Some("+"),
      Self::Star => Some("*"),
    }
  }

  pub(crate) fn is_variadic(self) -> bool {
    self != Self::Singular
  }
}
