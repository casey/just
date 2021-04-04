use crate::common::*;

#[derive(Clone, Copy)]
pub(crate) enum StringKind {
  Backtick,
  Cooked,
  Raw,
}

impl StringKind {
  pub(crate) fn delimiter(self) -> char {
    match self {
      Self::Backtick => '`',
      Self::Cooked => '"',
      Self::Raw => '\'',
    }
  }

  pub(crate) fn token_kind(self) -> TokenKind {
    match self {
      Self::Backtick => TokenKind::Backtick,
      Self::Cooked => TokenKind::StringCooked,
      Self::Raw => TokenKind::StringRaw,
    }
  }
}
