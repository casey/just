use crate::common::*;

/// A line fragment consisting either of…
#[derive(PartialEq, Debug)]
pub(crate) enum Fragment<'src> {
  /// …raw text…
  Text { token: Token<'src> },
  /// …an interpolation containing `expression`.
  Interpolation { expression: Expression<'src> },
}

impl<'src> Display for Fragment<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match self {
      Self::Text { token } => write!(f, "{}", token.lexeme()),
      Self::Interpolation { expression } => write!(f, "{{{{ {} }}}}", expression),
    }
  }
}
