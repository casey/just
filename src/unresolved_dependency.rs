use super::*;

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct UnresolvedDependency<'src> {
  pub(crate) arguments: Vec<Expression<'src>>,
  pub(crate) recipe: Namepath<'src>,
}

impl Display for UnresolvedDependency<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.arguments.is_empty() {
      write!(f, "{}", self.recipe)
    } else {
      write!(f, "({}", self.recipe)?;

      for argument in &self.arguments {
        write!(f, " {argument}")?;
      }

      write!(f, ")")
    }
  }
}
