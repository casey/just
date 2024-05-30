use super::*;

#[derive(Default, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Namepath<'src>(Vec<Name<'src>>);

impl<'src> Namepath<'src> {
  pub(crate) fn join(&self, name: Name<'src>) -> Self {
    Self(self.0.iter().copied().chain(iter::once(name)).collect())
  }

  pub(crate) fn spaced(&self) -> ModulePath {
    ModulePath {
      path: self.0.iter().map(|name| name.lexeme().into()).collect(),
      spaced: true,
    }
  }
}

impl<'src> Display for Namepath<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for (i, name) in self.0.iter().enumerate() {
      if i > 0 {
        write!(f, "::")?;
      }
      write!(f, "{name}")?;
    }
    Ok(())
  }
}

impl<'src> Serialize for Namepath<'src> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&format!("{self}"))
  }
}
