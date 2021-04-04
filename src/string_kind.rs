use crate::common::*;

#[derive(Clone, Copy, Debug, PartialEq)]
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

  pub(crate) fn unterminated_error_kind(self) -> CompilationErrorKind<'static> {
    CompilationErrorKind::UnterminatedString(self)
  }

  pub(crate) fn processes_escape_sequences(self) -> bool {
    match self {
      Self::Backtick | Self::Raw => false,
      Self::Cooked => true,
    }
  }
}
