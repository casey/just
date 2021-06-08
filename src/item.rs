use crate::common::*;

/// A single top-level item
#[derive(Debug, Clone)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src, Name<'src>>),
  Assignment(Assignment<'src>),
  Comment(&'src str),
  Recipe(UnresolvedRecipe<'src>),
  Set(Set<'src>),
}

impl<'src> Display for Item<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Item::Alias(alias) => write!(f, "{}", alias),
      Item::Assignment(assignment) => write!(f, "{}", assignment),
      Item::Comment(comment) => write!(f, "{}", comment),
      Item::Recipe(recipe) => write!(f, "{}", recipe),
      Item::Set(set) => write!(f, "{}", set),
    }
  }
}
