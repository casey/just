use super::*;

// todo:
// - UserFunction -> Function
// - Function -> ?
// - combine tests
//
// - system function
// - standard function
// - function definition
// - builtin function
// - FunctionDefinition

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionDefinition<'src> {
  pub(crate) body: Expression<'src>,
  pub(crate) file_depth: u32,
  pub(crate) name: Name<'src>,
  pub(crate) parameters: Vec<(Name<'src>, Number)>,
}

impl<'src> Keyed<'src> for FunctionDefinition<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
