use super::*;

/// A binding of `name` to `value`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Binding<'src, V = String> {
  /// Export binding as an environment variable to child processes
  pub(crate) export: bool,
  /// Binding name
  pub(crate) name: Name<'src>,
  /// Binding value
  pub(crate) value: V,
}

// a subclass of Binding with "depth" field
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct DepthAwareBinding<'src, V = String> {
  /// Binding depth
  pub(crate) depth: u32,
  /// Export binding as an environment variable to child processes
  pub(crate) export: bool,
  /// Binding name
  pub(crate) name: Name<'src>,
  /// Binding value
  pub(crate) value: V,
}

impl<'src, V> Keyed<'src> for Binding<'src, V> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}


impl<'src, V> Keyed<'src> for DepthAwareBinding<'src, V> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}