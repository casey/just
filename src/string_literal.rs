use crate::common::*;

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct StringLiteral<'src> {
  pub(crate) kind: StringKind,
  pub(crate) raw: &'src str,
  pub(crate) cooked: String,
}

impl Display for StringLiteral<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}{}{}",
      self.kind.delimiter(),
      self.raw,
      self.kind.delimiter()
    )
  }
}

impl<'src> Serialize for StringLiteral<'src> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.cooked)
  }
}
