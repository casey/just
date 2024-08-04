use super::*;

/// A binding of `name` to `value`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Binding<'src, V = String> {
  /// Module depth where binding appears
  pub(crate) depth: u32,
  /// Export binding as an environment variable to child processes
  pub(crate) export: bool,
  /// Binding name
  pub(crate) name: Name<'src>,
  /// Whether this binding is private to the script
  pub(crate) private: bool,
  /// Binding value
  pub(crate) value: V,
}

impl<V> Binding<'_, V> {
  pub fn is_public(&self) -> bool {
    !self.private
  }
}

impl<'src, V> Keyed<'src> for Binding<'src, V> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
