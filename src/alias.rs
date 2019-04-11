use crate::common::*;

pub struct Alias<'a> {
  pub name: &'a str,
  pub target: &'a str,
  pub line_number: usize,
  pub private: bool,
}

impl<'a> Display for Alias<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "alias {} = {}", self.name, self.target)
  }
}
