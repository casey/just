use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialOrd)]
pub(crate) struct Namepath<'src>(Vec<Name<'src>>);

impl PartialEq for Namepath<'_> {
  fn eq(&self, other: &Self) -> bool {
    let self_lexeme_iter = self.lexeme_iter();
    let other_lexeme_iter = other.lexeme_iter();
    if self_lexeme_iter.len() != other_lexeme_iter.len() {
      return false;
    }
    self_lexeme_iter
      .zip(other_lexeme_iter)
      .all(|(self_lexeme, other_lexeme)| self_lexeme == other_lexeme)
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

  fn lexeme_iter(&self) -> impl ExactSizeIterator<Item = &str> {
    self.0.iter().map(|name| name.lexeme())
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

impl<'src> Deref for Namepath<'src> {
  type Target = [Name<'src>];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
