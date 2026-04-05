use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionDefinition<'src> {
  pub(crate) body: Expression<'src>,
  pub(crate) name: Name<'src>,
  pub(crate) parameters: Vec<(Name<'src>, Number)>,
}

impl<'src> Keyed<'src> for FunctionDefinition<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
