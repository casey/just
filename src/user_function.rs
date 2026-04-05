use super::*;

// todo:
// - UserFunction -> Function
// - Function -> ?
// - can functions even appear in settings?
//
// later:
// - allow functions in settings

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UserFunction<'src> {
  pub(crate) body: Expression<'src>,
  pub(crate) file_depth: u32,
  pub(crate) name: Name<'src>,
  pub(crate) number: Number,
  pub(crate) parameters: Vec<Name<'src>>,
}

impl<'src> Keyed<'src> for UserFunction<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
