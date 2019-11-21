use crate::common::*;

/// An alias, e.g. `name := target`
#[derive(Debug, PartialEq)]
pub(crate) struct Alias<'src, T = Rc<Recipe<'src>>> {
  pub(crate) name: Name<'src>,
  pub(crate) target: T,
}

impl<'src> Alias<'src, Name<'src>> {
  pub(crate) fn line_number(&self) -> usize {
    self.name.line
  }

  pub(crate) fn resolve(self, target: Rc<Recipe<'src>>) -> Alias<'src> {
    assert_eq!(self.target.lexeme(), target.name.lexeme());

    Alias {
      name: self.name,
      target,
    }
  }
}

impl Alias<'_> {
  pub(crate) fn is_private(&self) -> bool {
    self.name.lexeme().starts_with('_')
  }
}

impl<'src, T> Keyed<'src> for Alias<'src, T> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}

impl<'src> Display for Alias<'src, Name<'src>> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "alias {} := {}",
      self.name.lexeme(),
      self.target.lexeme()
    )
  }
}

impl<'src> Display for Alias<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "alias {} := {}",
      self.name.lexeme(),
      self.target.name.lexeme()
    )
  }
}
