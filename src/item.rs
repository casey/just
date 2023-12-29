use super::*;

/// A single top-level item
#[derive(Debug, Clone)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src, Name<'src>>),
  Assignment(Assignment<'src>),
  Comment(&'src str),
  Import {
    absolute: Option<PathBuf>,
    relative: StringLiteral<'src>,
  },
  Module {
    absolute: Option<PathBuf>,
    name: Name<'src>,
    path: Option<StringLiteral<'src>>,
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
      Item::Module { name, path, .. } => {
        write!(f, "mod {name}")?;

        if let Some(path) = path {
          write!(f, " {path}")?;
        }

        Ok(())
      }
      Item::Recipe(recipe) => write!(f, "{}", recipe.color_display(Color::never())),
      Item::Set(set) => write!(f, "{set}"),
    }
  }
}
