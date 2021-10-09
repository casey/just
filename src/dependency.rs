use crate::common::*;

#[derive(PartialEq, Debug, Serialize)]
pub(crate) struct Dependency<'src> {
  pub(crate) arguments: Vec<Expression<'src>>,
  #[serde(rename = "name", serialize_with = "serialize_name")]
  pub(crate) recipe: Rc<Recipe<'src>>,
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

fn serialize_name<S>(recipe: &Rc<Recipe>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  serializer.serialize_str(&recipe.name())
}
