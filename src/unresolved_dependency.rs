use super::*;

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct UnresolvedDependency<'src> {
  pub(crate) arguments: Vec<DependencyArgument<'src>>,
  pub(crate) recipe: Namepath<'src>,
}

impl UnresolvedDependency<'_> {
  pub(crate) fn starred(&self) -> bool {
    self.arguments.iter().any(|argument| argument.starred)
  }
}

impl Display for UnresolvedDependency<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.arguments.is_empty() {
      write!(f, "{}", self.recipe)
    } else {
      if self.starred() {
        write!(f, "*")?;
      }

      write!(f, "({}", self.recipe)?;

      for argument in &self.arguments {
        write!(f, " {argument}")?;
      }

      write!(f, ")")
    }
  }
}
