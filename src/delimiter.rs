use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Delimiter {
  Brace,
  Bracket,
  FormatString(StringKind),
  Paren,
}

impl Delimiter {
  pub(crate) fn open(self) -> char {
    match self {
      Self::Brace | Self::FormatString(_) => '{',
      Self::Bracket => '[',
      Self::Paren => '(',
    }
  }

  pub(crate) fn close(self) -> char {
    match self {
      Self::Brace | Self::FormatString(_) => '}',
      Self::Bracket => ']',
      Self::Paren => ')',
    }
  }
}
