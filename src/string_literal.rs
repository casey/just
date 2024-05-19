use super::*;

#[derive(PartialEq, Debug, Clone, Ord, Eq, PartialOrd)]
pub(crate) struct StringLiteral<'src> {
  pub(crate) cooked: String,
  pub(crate) expand: bool,
  pub(crate) kind: StringKind,
  pub(crate) raw: &'src str,
}

impl Display for StringLiteral<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.expand {
      write!(f, "x")?;
    }

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
