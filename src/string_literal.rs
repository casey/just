use super::*;

#[derive(PartialEq, Debug, Clone, Eq, PartialOrd)]
pub(crate) struct StringLiteral<'src> {
  pub(crate) cooked: String,
  pub(crate) expand: bool,
  pub(crate) kind: StringKind,
  pub(crate) raw: &'src str,
  pub(crate) name: Option<Name<'src>>,
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
      raw,
      name: None,
    }
  }
}

impl<'src> cmp::Ord for StringLiteral<'src> {
  #[inline]
  fn cmp(&self, other: &StringLiteral) -> cmp::Ordering {
    match Ord::cmp(&self.cooked, &other.cooked) {
      Ordering::Equal => match Ord::cmp(&self.expand, &other.expand) {
        Ordering::Equal => match Ord::cmp(&self.kind, &other.kind) {
          Ordering::Equal => Ord::cmp(&self.raw, &other.raw),
          cmp => cmp,
        },
        cmp => cmp,
      },
      cmp => cmp,
    }
  }
}

impl<'src> Display for StringLiteral<'src> {
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
