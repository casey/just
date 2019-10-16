use crate::common::*;

/// An assignment, e.g `foo := bar`
#[derive(Debug, PartialEq)]
pub(crate) struct Assignment<'src> {
  /// Assignment was prefixed by the `export` keyword
  pub(crate) export: bool,
  /// Left-hand side of the assignment
  pub(crate) name: Name<'src>,
  /// Right-hand side of the assignment
  pub(crate) expression: Expression<'src>,
}

impl<'src> Keyed<'src> for Assignment<'src> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
