use super::*;

#[derive(PartialEq, Debug, Serialize)]
pub(crate) struct Dependency<'src> {
  #[serde(serialize_with = "flatten_arguments")]
  pub(crate) arguments: Vec<Vec<Expression<'src>>>,
  #[serde(serialize_with = "keyed::serialize")]
  pub(crate) recipe: Arc<Recipe<'src>>,
}

fn flatten_arguments<S: Serializer>(
  arguments: &[Vec<Expression<'_>>],
  serializer: S,
) -> Result<S::Ok, S::Error> {
  let len = arguments.iter().map(Vec::len).sum();
  let mut seq = serializer.serialize_seq(Some(len))?;
  for group in arguments {
    for argument in group {
      seq.serialize_element(argument)?;
    }
  }
  seq.end()
}

impl Display for Dependency<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.arguments.is_empty() {
      write!(f, "{}", self.recipe.name())
    } else {
      write!(f, "({}", self.recipe.name())?;

      for group in &self.arguments {
        for argument in group {
          write!(f, " {argument}")?;
        }
      }

      write!(f, ")")
    }
  }
}
