use crate::common::*;

/// A single line in a recipe body, consisting of any number of `Fragment`s.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Line<'src> {
  pub(crate) fragments: Vec<Fragment<'src>>,
}

impl<'src> Line<'src> {
  pub(crate) fn is_empty(&self) -> bool {
    self.fragments.is_empty()
  }

  pub(crate) fn is_continuation(&self) -> bool {
    match self.fragments.last() {
      Some(Fragment::Text { token }) => token.lexeme().ends_with('\\'),
      _ => false,
    }
  }

  pub(crate) fn is_shebang(&self) -> bool {
    match self.fragments.first() {
      Some(Fragment::Text { token }) => token.lexeme().starts_with("#!"),
      _ => false,
    }
  }

  pub(crate) fn is_quiet(&self) -> bool {
    match self.fragments.first() {
      Some(Fragment::Text { token }) => token.lexeme().starts_with('@'),
      _ => false,
    }
  }

  pub(crate) fn is_infallable(&self) -> bool {
    match self.fragments.first() {
      Some(Fragment::Text { token }) => token.lexeme().starts_with('-'),
      _ => false,
    }
  }
}
