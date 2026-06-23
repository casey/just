use super::*;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ModuleAlias<'src> {
  pub(crate) attributes: AttributeSet<'src>,
  pub(crate) name: Name<'src>,
  pub(crate) target: Modulepath,
}

impl<'src> Keyed<'src> for ModuleAlias<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
