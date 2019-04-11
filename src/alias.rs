use common::*;

pub struct Alias<'a> {
  pub name: &'a str,
  pub target: &'a str,
  pub line_number: usize,
}

impl<'a> Display for Alias<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "alias {} = {}", self.name, self.target)
  }
}
