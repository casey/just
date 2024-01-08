use super::*;

#[derive(Default, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Namepath<'src>(Vec<Name<'src>>);

impl<'src> Namepath<'src> {
  pub(crate) fn join(&self, name: Name<'src>) -> Self {
    Self(self.0.iter().cloned().chain(iter::once(name)).collect())
  }
}

impl<'str> Serialize for Namepath<'str> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut path = String::new();

    for (i, name) in self.0.iter().enumerate() {
      if i > 0 {
        path.push_str("::");
      }
      path.push_str(&name.lexeme());
    }

    serializer.serialize_str(&path)
  }
}
