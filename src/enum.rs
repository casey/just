use super::*;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub(crate) struct Enum<'src> {
  pub name: Name<'src>,
  pub variants: BTreeMap<&'src str, StringLiteral<'src>>,
}

impl Display for Enum<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "enum {}", self.name)
  }
}

impl<'src> Keyed<'src> for Enum<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
