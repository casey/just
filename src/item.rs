use crate::common::*;

/// A single top-level item
#[derive(Debug)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src>),
  Assignment(Assignment<'src>),
  Recipe(Recipe<'src>),
}
