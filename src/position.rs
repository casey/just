/// Source position
#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) struct Position {
  pub(crate) offset: usize,
  pub(crate) column: usize,
  pub(crate) line:   usize,
}
