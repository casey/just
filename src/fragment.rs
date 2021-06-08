use crate::common::*;

/// A line fragment consisting either of…
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Fragment<'src> {
  /// …raw text…
  Text { token: Token<'src> },
  /// …an interpolation containing `expression`.
  Interpolation { expression: Expression<'src> },
}
