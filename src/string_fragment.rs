use crate::common::*;

#[derive(PartialEq, Debug)]
/// Similar in purpose to `Fragment`, but for interpolated strings/backticks instead of recipe
/// bodies.  The difference is that `StringFragment` deals with escape sequences/cooking,
/// un-indenting, etc., while `Fragment` does not.
pub(crate) enum StringFragment<'src> {
  /// …raw text…
  Text { raw: &'src str, cooked: String },
  /// …an interpolation containing `expression`.
  Interpolation { expression: Expression<'src> },
}

impl<'src> Display for StringFragment<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match self {
      Self::Text { raw, .. } => write!(f, "{}", raw),
      Self::Interpolation { expression } => write!(f, "{{{{ {} }}}}", expression),
    }
  }
}
