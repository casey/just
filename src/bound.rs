use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(crate) struct Bound {
  pub(crate) max: Option<usize>,
  pub(crate) min: usize,
}
