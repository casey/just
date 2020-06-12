use crate::common::*;

/// Parameters can either be…
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ParameterKind {
  /// …singular, accepting a single argument
  Singular,
  /// …variadic, accepting one or more arguments
  VariadicOneOrMore,
  /// …variadic, accepting zero or more arguments
  VariadicZeroOrMore,
}

impl ParameterKind {
  pub(crate) fn as_str(self) -> &'static str {
    match self {
      Self::Singular => "",
      Self::VariadicOneOrMore => "+",
      Self::VariadicZeroOrMore => "*",
    }
  }

  pub(crate) fn is_variadic(self) -> bool {
    self != Self::Singular
  }
}
