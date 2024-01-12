use super::*;

/// A name. This is just a `Token` of kind `Identifier`, but we give it its own
/// type for clarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) struct Name<'src> {
  pub(crate) token: Token<'src>,
}

impl<'src> Name<'src> {
  pub(crate) fn from_identifier(token: Token<'src>) -> Self {
    assert_eq!(token.kind, TokenKind::Identifier);
    Self { token }
  }
}

impl<'src> Deref for Name<'src> {
  type Target = Token<'src>;

  fn deref(&self) -> &Self::Target {
    &self.token
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
