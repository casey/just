use crate::common::*;

#[derive(Debug, PartialEq, Clone, Copy, Ord, PartialOrd, Eq)]
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
    TokenKind::StringToken(self)
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
