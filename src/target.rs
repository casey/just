use super::*;

pub(crate) enum Target<'src> {
  Name(Name<'src>),
  Path { path: String, token: Token<'src> },
}

impl<'src> Target<'src> {
  pub(crate) fn token(&self) -> Token<'src> {
    match self {
      Self::Name(name) => name.token(),
      Self::Path { path, token } => *token,
    }
  }
}

impl<'src> Serialize for Target<'src> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::Name(name) => name.serialize(serializer),
      Self::Path { path, token } => *token,
    }
  }
}
