use super::*;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub(crate) struct Enum<'src> {
  pub name: Name<'src>,
  pub variants: BTreeMap<&'src str, StringLiteral<'src>>,
}

impl Display for Enum<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "enum {} := [ ", self.name)?;
    let mut first = true;
    for (key, value) in &self.variants {
      if !first {
        write!(f, ", ")?;
      }
      write!(f, "\"{}\" = \"{}\"", key, value.cooked)?;
      first = false;
    }
    write!(f, " ]")
  }
}

impl<'src> Keyed<'src> for Enum<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
