use crate::common::*;

#[derive(Debug, PartialEq, Clone, Copy, Ord, PartialOrd, Eq)]
pub(crate) enum StringKind {
  Backtick,
  BacktickMultiline,
  Cooked,
  CookedMultiline,
  Raw,
  RawMultiline,
}

impl StringKind {
  const ALL: &'static [Self] = &[
    Self::BacktickMultiline,
    Self::Backtick,
    Self::CookedMultiline,
    Self::Cooked,
    Self::RawMultiline,
    Self::Raw,
  ];

  pub(crate) fn delimiter(self) -> &'static str {
    match self {
      Self::Backtick => "`",
      Self::BacktickMultiline => "```",
      Self::Cooked => "\"",
      Self::CookedMultiline => "\"\"\"",
      Self::Raw => "'",
      Self::RawMultiline => "'''",
    }
  }

  pub(crate) fn delimiter_len(self) -> usize {
    self.delimiter().len()
  }

  pub(crate) fn token_kind(self) -> TokenKind {
    match self {
      Self::Backtick | Self::BacktickMultiline => TokenKind::Backtick,
      Self::Cooked | Self::CookedMultiline | Self::Raw | Self::RawMultiline =>
        TokenKind::StringToken,
    }
  }

  pub(crate) fn unterminated_error_kind(self) -> CompilationErrorKind<'static> {
    CompilationErrorKind::UnterminatedString(self)
  }

  pub(crate) fn processes_escape_sequences(self) -> bool {
    match self {
      Self::Backtick | Self::BacktickMultiline | Self::Raw | Self::RawMultiline => false,
      Self::Cooked | Self::CookedMultiline => true,
    }
  }

  pub(crate) fn indented(self) -> bool {
    match self {
      Self::BacktickMultiline | Self::CookedMultiline | Self::RawMultiline => true,
      Self::Backtick | Self::Cooked | Self::Raw => false,
    }
  }

  pub(crate) fn from_string_or_backtick<'src>(token: Token<'src>) -> CompilationResult<'src, Self> {
    Self::from_token_start(token.lexeme()).ok_or_else(|| {
      token.error(CompilationErrorKind::Internal {
        message: "StringKind::from_token: Expected String or Backtick".to_owned(),
      })
    })
  }

  pub(crate) fn from_token_start(token_start: &str) -> Option<Self> {
    for &kind in Self::ALL {
      if token_start.starts_with(kind.delimiter()) {
        return Some(kind);
      }
    }

    None
  }
}
