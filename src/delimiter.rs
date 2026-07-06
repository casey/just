use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Delimiter {
  Brace,
  Bracket,
  FormatString(StringKind),
  Paren,
}

impl Delimiter {
  pub(crate) fn open(self) -> &'static str {
    match self {
      Self::Brace => "{",
      Self::Bracket => "[",
      Self::FormatString(_) => "{{",
      Self::Paren => "(",
    }
  }

  pub(crate) fn close(self) -> &'static str {
    match self {
      Self::Brace => "}",
      Self::Bracket => "]",
      Self::FormatString(_) => "}}",
      Self::Paren => ")",
    }
  }
}
