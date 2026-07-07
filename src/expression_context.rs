use super::*;

#[derive(Default)]
pub(crate) struct ExpressionContext<'src> {
  bindings: HashMap<&'src str, Number>,
}

impl ExpressionContext<'_> {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn shadows(&self, name: &str) -> bool {
    self.bindings.contains_key(name)
  }
}

impl<'src> From<&[(Name<'src>, Number)]> for ExpressionContext<'src> {
  fn from(parameters: &[(Name<'src>, Number)]) -> Self {
    Self {
      bindings: parameters
        .iter()
        .map(|(name, number)| (name.lexeme(), *number))
        .collect(),
    }
  }
}

impl<'src> From<&[Parameter<'src>]> for ExpressionContext<'src> {
  fn from(parameters: &[Parameter<'src>]) -> Self {
    Self {
      bindings: parameters
        .iter()
        .map(|parameter| (parameter.name.lexeme(), parameter.number))
        .collect(),
    }
  }
}
