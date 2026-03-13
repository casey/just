use super::*;

/// A line fragment consisting either of…
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Fragment<'src> {
  /// …an interpolation containing `expression`.
  Interpolation { expression: Expression<'src> },
  /// …raw text…
  Text { token: Token<'src> },
}

impl Serialize for Fragment<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::Text { token } => serializer.serialize_str(token.lexeme()),
      Self::Interpolation { expression } => {
        let mut seq = serializer.serialize_seq(None)?;
        seq.serialize_element(expression)?;
        seq.end()
      }
    }
  }
}
