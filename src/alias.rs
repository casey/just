use crate::common::*;

#[derive(Debug)]
pub(crate) struct Alias<'a> {
  pub(crate) name: &'a str,
  pub(crate) target: &'a str,
  pub(crate) line_number: usize,
  pub(crate) private: bool,
}

impl<'a> Display for Alias<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "alias {} := {}", self.name, self.target)
  }
}
