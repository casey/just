use super::*;

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Dependency<'src> {
  pub(crate) arguments: Vec<Vec<DependencyArgument<'src>>>,
  pub(crate) path: Namepath<'src>,
  pub(crate) recipe: Arc<Recipe<'src>>,
}

impl Dependency<'_> {
  pub(crate) fn star(&self) -> Option<usize> {
    self
      .arguments
      .iter()
      .position(|group| group.iter().any(|argument| argument.starred))
  }
}

impl Serialize for Dependency<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let arguments = self
      .arguments
      .iter()
      .flatten()
      .map(|argument| &argument.expression)
      .collect::<Vec<&Expression>>();

    let star = self
      .arguments
      .iter()
      .flatten()
      .position(|argument| argument.starred);

    let mut s = serializer.serialize_struct("Dependency", 3)?;
    s.serialize_field("arguments", &arguments)?;
    s.serialize_field("recipe", &self.path)?;
    s.serialize_field("star", &star)?;
    s.end()
  }
}

impl Display for Dependency<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.arguments.is_empty() {
      write!(f, "{}", self.path)
    } else {
      if self.star().is_some() {
        write!(f, "*")?;
      }

      write!(f, "({}", self.path)?;

      for argument in self.arguments.iter().flatten() {
        write!(f, " {argument}")?;
      }

      write!(f, ")")
    }
  }
}
