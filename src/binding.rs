use crate::common::*;

/// A binding of `name` to `value`
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Binding<'src, V = String, C = bool> {
  /// Export binding only if conditional is true
  pub(crate) condition: Option<C>,
  /// Export binding as an environment variable to child processes
  pub(crate) export:    bool,
  /// Binding name
  pub(crate) name:      Name<'src>,
  /// Binding value
  pub(crate) value:     V,
}

impl<'src, V, C> Keyed<'src> for Binding<'src, V, C> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
