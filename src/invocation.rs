use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Invocation<'src, 'run> {
  pub(crate) arguments: Vec<Value>,
  pub(crate) recipe: &'run Recipe<'src>,
}
