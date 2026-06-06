use super::*;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct Disabled<'src> {
  pub(crate) modules: BTreeSet<Modulepath>,
  pub(crate) name: Name<'src>,
}

impl<'src> Keyed<'src> for Disabled<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
