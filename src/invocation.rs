use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct Invocation<T> {
  pub(crate) arguments: Vec<Vec<String>>,
  pub(crate) target: T,
}
