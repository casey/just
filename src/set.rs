use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Set<'src> {
  pub(crate) name: Name<'src>,
  pub(crate) value: Setting<'src>,
}

impl<'src> Keyed<'src> for Set<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}

impl<'src> Display for Set<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "set {} := {}", self.name, self.value)
  }
}
