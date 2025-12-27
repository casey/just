use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Invocation<'src, 'run> {
  pub(crate) arguments: Vec<Vec<String>>,
  pub(crate) recipe: &'run Recipe<'src>,
}
