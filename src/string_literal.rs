use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) struct StringLiteral<'src> {
  pub(crate) raw: &'src str,
  pub(crate) cooked: Cow<'src, str>,
}

impl Display for StringLiteral<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.cooked {
      Cow::Borrowed(raw) => write!(f, "'{}'", raw),
      Cow::Owned(_) => write!(f, "\"{}\"", self.raw),
    }
  }
}
