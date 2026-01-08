use super::*;

#[derive(PartialEq, Debug, Clone, Ord, Eq, PartialOrd)]
pub(crate) struct StringLiteral<'src> {
  pub(crate) cooked: String,
  pub(crate) expand: bool,
  pub(crate) kind: StringKind,
  pub(crate) part: Option<FormatStringPart>,
  pub(crate) token: Token<'src>,
}

impl Display for StringLiteral<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.expand {
      write!(f, "x")?;
    }

    if let Some(FormatStringPart::Start | FormatStringPart::Single) = self.part {
      write!(f, "f")?;
    }

    write!(f, "{}", self.token.lexeme())
  }
}

impl Serialize for StringLiteral<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.cooked)
  }
}
