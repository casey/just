use crate::common::*;

/// A single top-level item
#[derive(Debug)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src, Name<'src>>),
  Assignment(Assignment<'src>),
  Recipe(Recipe<'src, Name<'src>>),
  Set(Set<'src>),
}
