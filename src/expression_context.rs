use super::*;

#[derive(Default)]
pub(crate) struct ExpressionContext<'src> {
  bindings: HashMap<&'src str, Number>,
}

impl ExpressionContext<'_> {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn lookup(&self, name: &str) -> Option<Number> {
    self.bindings.get(name).copied()
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
