use crate::common::*;

/// A line fragment consisting either of…
#[derive(PartialEq, Debug)]
pub(crate) enum Fragment<'src> {
  /// …raw text…
  Text { token: Token<'src> },
  /// …an interpolation containing `expression`.
  Interpolation { expression: Expression<'src> },
}
