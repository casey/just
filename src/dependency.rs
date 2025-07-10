use super::*;

#[derive(PartialEq, Debug, Serialize)]
pub(crate) struct Dependency<'src> {
  pub(crate) arguments: Vec<Expression<'src>>,
  #[serde(serialize_with = "keyed::serialize")]
  pub(crate) recipe: Arc<Recipe<'src>>,
}

impl Display for Dependency<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.arguments.is_empty() {
      write!(f, "{}", self.recipe.name())
    } else {
      write!(f, "({}", self.recipe.name())?;

      for argument in &self.arguments {
        write!(f, " {argument}")?;
      }

      write!(f, ")")
    }
  }
}
