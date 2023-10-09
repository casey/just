use super::*;

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct UnresolvedDependency<'src> {
  pub(crate) recipe: Name<'src>,
  pub(crate) arguments: Vec<Expression<'src>>,
}

impl<'src> Display for UnresolvedDependency<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
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
