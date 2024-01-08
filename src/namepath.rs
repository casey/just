use super::*;

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct Namepath<'src>(Vec<Name<'src>>);

impl<'src> Namepath<'src> {
  pub(crate) fn join(&self, name: Name<'src>) -> Self {
    Self(self.0.iter().cloned().chain(iter::once(name)).collect())
  }
}
