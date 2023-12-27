use super::*;

/// A single top-level item
#[derive(Debug, Clone)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src, Name<'src>>),
  Assignment(Assignment<'src>),
  Comment(&'src str),
  Import {
    relative: StringLiteral<'src>,
    absolute: Option<PathBuf>,
  },
  Mod {
    name: Name<'src>,
    absolute: Option<PathBuf>,
  },
  Recipe(UnresolvedRecipe<'src>),
  Set(Set<'src>),
}

impl<'src> Display for Item<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Item::Alias(alias) => write!(f, "{alias}"),
      Item::Assignment(assignment) => write!(f, "{assignment}"),
      Item::Comment(comment) => write!(f, "{comment}"),
      Item::Import { relative, .. } => write!(f, "import {relative}"),
      Item::Mod { name, .. } => write!(f, "mod {name}"),
      Item::Recipe(recipe) => write!(f, "{}", recipe.color_display(Color::never())),
      Item::Set(set) => write!(f, "{set}"),
    }
  }
}
