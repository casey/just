use super::*;

/// A binding of `name` to `value`
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct Binding<'src, V = String> {
  #[serde(skip)]
  pub(crate) constant: bool,
  pub(crate) export: bool,
  #[serde(skip)]
  pub(crate) file_depth: u32,
  pub(crate) lazy: bool,
  pub(crate) name: Name<'src>,
  pub(crate) private: bool,
  pub(crate) value: V,
}

impl<'src, V> Keyed<'src> for Binding<'src, V> {
  fn key(&self) -> &'src str {
    self.name.lexeme()
  }
}
