/// Source position
#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) struct Position {
  pub(crate) column: usize,
  pub(crate) line: usize,
  pub(crate) offset: usize,
}
