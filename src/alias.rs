use crate::common::*;

/// An alias, e.g. `name := target`
#[derive(Debug, PartialEq)]
pub(crate) struct Alias<'src> {
  pub(crate) name: Name<'src>,
  pub(crate) target: Name<'src>,
}

impl Alias<'_> {
  pub(crate) fn is_private(&self) -> bool {
    self.name.lexeme().starts_with('_')
  }

  pub(crate) fn line_number(&self) -> usize {
    self.name.line
  }
}

impl<'src> Keyed<'src> for Alias<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}

impl<'a> Display for Alias<'a> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "alias {} := {}",
      self.name.lexeme(),
      self.target.lexeme()
    )
  }
}
