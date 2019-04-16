/// Source position
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Position {
  pub offset: usize,
  pub column: usize,
  pub line: usize,
}
