use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) struct Dependency<'src> {
  pub(crate) recipe:    Rc<Recipe<'src>>,
  pub(crate) arguments: Vec<Expression<'src>>,
}

impl<'src> Display for Dependency<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    if self.arguments.is_empty() {
      write!(f, "{}", self.recipe.name())
    } else {
      write!(f, "({}", self.recipe.name())?;

      for argument in &self.arguments {
        write!(f, " {}", argument)?;
      }

      write!(f, ")")
    }
  }
}
