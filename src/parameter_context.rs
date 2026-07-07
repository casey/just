use super::*;

#[derive(Clone, Copy)]
pub(crate) enum ParameterContext<'a, 'src> {
  Function(&'a [(Name<'src>, Number)]),
  None,
  Recipe(&'a [Parameter<'src>]),
}

impl ParameterContext<'_, '_> {
  pub(crate) fn shadows(self, name: &str) -> bool {
    match self {
      Self::Function(parameters) => parameters
        .iter()
        .any(|(parameter, _number)| parameter.lexeme() == name),
      Self::None => false,
      Self::Recipe(parameters) => parameters
        .iter()
        .any(|parameter| parameter.name.lexeme() == name),
    }
  }
}
