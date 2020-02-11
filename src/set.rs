use crate::common::*;

#[derive(Debug)]
pub(crate) struct Set<'src> {
  pub(crate) name:  Name<'src>,
  pub(crate) value: Setting<'src>,
}

impl<'src> Keyed<'src> for Set<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
