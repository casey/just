use super::*;

#[derive(PartialEq, Debug, Clone, Ord, Eq, PartialOrd)]
pub(crate) struct StringLiteral<'src> {
  pub(crate) cooked: String,
  pub(crate) expand: bool,
  pub(crate) kind: StringKind,
  pub(crate) part: Option<FormatStringPart>,
  pub(crate) raw: &'src str,
}

impl<'src> StringLiteral<'src> {
  pub(crate) fn from_raw(raw: &'src str) -> Self {
    Self {
      cooked: raw.into(),
      expand: false,
      kind: StringKind {
        delimiter: StringDelimiter::QuoteSingle,
        indented: false,
      },
      part: None,
      raw,
    }
  }
}

impl Display for StringLiteral<'_> {
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
      INTERPOLATION_CLOSE
    } else {
      self.kind.delimiter()
    };

    let close = if matches!(
      self.part,
      Some(FormatStringPart::Start | FormatStringPart::Continue)
    ) {
      INTERPOLATION_OPEN
    } else {
      self.kind.delimiter()
    };

    write!(f, "{open}{}{close}", self.raw)
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
