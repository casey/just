use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
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

  pub(crate) fn push(&mut self, name: Name<'src>) {
    self.0.push(name);
  }

  pub(crate) fn last(&self) -> &Name<'src> {
    self.0.last().unwrap()
  }

  pub(crate) fn split_last(&self) -> (&Name<'src>, &[Name<'src>]) {
    self.0.split_last().unwrap()
  }

  #[cfg(test)]
  pub(crate) fn iter(&self) -> slice::Iter<'_, Name<'src>> {
    self.0.iter()
  }

  pub(crate) fn len(&self) -> usize {
    self.0.len()
  }
}

impl Display for Namepath<'_> {
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

impl<'src> From<Name<'src>> for Namepath<'src> {
  fn from(name: Name<'src>) -> Self {
    Self(vec![name])
  }
}

impl Serialize for Namepath<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&format!("{self}"))
  }
}
