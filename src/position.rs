/// Source position
#[derive(Copy, Clone)]
pub struct Position {
  pub offset: usize,
  pub column: usize,
  pub line: usize,
}
