use super::*;

#[derive(PartialEq, Debug, Clone, Ord, Eq, PartialOrd)]
pub(crate) struct StringLiteral {
  pub(crate) cooked: String,
  pub(crate) expand: bool,
  pub(crate) kind: StringKind,
  pub(crate) part: Option<FormatStringPart>,
  pub(crate) raw: String,
}

impl Display for StringLiteral {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.expand {
      write!(f, "x")?;
    }

    if let Some(FormatStringPart::Start | FormatStringPart::Single) = self.part {
      write!(f, "f")?;
    }

    let open = if matches!(
      self.part,
      Some(FormatStringPart::Continue | FormatStringPart::End)
    ) {
      Lexer::INTERPOLATION_END
    } else {
      self.kind.delimiter()
    };

    let close = if matches!(
      self.part,
      Some(FormatStringPart::Start | FormatStringPart::Continue)
    ) {
      Lexer::INTERPOLATION_START
    } else {
      self.kind.delimiter()
    };

    write!(f, "{open}{}{close}", self.raw)
  }
}

impl Serialize for StringLiteral {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.cooked)
  }
}
