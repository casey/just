use crate::common::*;

/// A name. This is effectively just a `Token` of kind `Identifier`, but we give
/// it its own type for clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) struct Name<'src> {
  pub(crate) offset: usize,
  pub(crate) length: usize,
  pub(crate) line: usize,
  pub(crate) column: usize,
  pub(crate) src: &'src str,
}

impl<'src> Name<'src> {
  /// The name's text contents
  pub(crate) fn lexeme(&self) -> &'src str {
    &self.src[self.offset..self.offset + self.length]
  }

  /// Turn this name back into a token
  pub(crate) fn token(&self) -> Token<'src> {
    Token {
      kind: TokenKind::Identifier,
      offset: self.offset,
      length: self.length,
      line: self.line,
      column: self.column,
      src: self.src,
    }
  }

  pub(crate) fn from_identifier(token: Token<'src>) -> Name {
    assert_eq!(token.kind, TokenKind::Identifier);
    Name {
      offset: token.offset,
      length: token.length,
      line: token.line,
      column: token.column,
      src: token.src,
    }
  }

  pub(crate) fn error(&self, kind: CompileErrorKind<'src>) -> CompileError<'src> {
    self.token().error(kind)
  }
}

impl Display for Name<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.lexeme())
  }
}

impl<'src> Serialize for Name<'src> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.lexeme())
  }
}
