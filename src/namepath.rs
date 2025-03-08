use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub(crate) struct Namepath<'src>(Vec<Name<'src>>);

impl Namepath<'_> {
  pub fn is_same_path(&self, other: &Self) -> bool {
    if self.len() != other.len() {
      return false;
    }

    self
      .iter()
      .zip(other.iter())
      .all(|(a, b)| a.lexeme() == b.lexeme())
  }
}

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

  pub fn push(&mut self, name: Name<'src>) {
    self.0.push(name);
  }

  pub fn iter(&self) -> std::slice::Iter<'_, Name<'src>> {
    self.0.iter()
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  /// Returns the last name in the module path
  /// a::b::c -> c
  /// a       -> a
  pub fn last(&self) -> &Name<'src> {
    self
      .0
      .last()
      .expect("Internal error: Namepath can not be empty")
  }

  /// Splits a module path into the last name and its parent
  /// a::b::c -> (a::b, c)
  /// a       -> (empty, a)
  pub fn split_last(&self) -> (&[Name<'src>], &Name<'src>) {
    let name = self.last();

    (&self.0[..self.0.len() - 1], name)
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
