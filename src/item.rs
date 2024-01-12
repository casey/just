use super::*;

/// A single top-level item
#[derive(Debug, Clone)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src, Name<'src>>),
  Assignment(Assignment<'src>),
  Comment(&'src str),
  Import {
    absolute: Option<PathBuf>,
    optional: bool,
    path: Token<'src>,
    relative: StringLiteral<'src>,
  },
  Module {
    absolute: Option<PathBuf>,
    name: Name<'src>,
    optional: bool,
    relative: Option<StringLiteral<'src>>,
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
      Item::Import {
        relative, optional, ..
      } => {
        write!(f, "import")?;

        if *optional {
          write!(f, "?")?;
        }

        write!(f, " {relative}")
      }
      Item::Module {
        name,
        relative,
        optional,
        ..
      } => {
        write!(f, "mod")?;

        if *optional {
          write!(f, "?")?;
        }

        write!(f, " {name}")?;

        if let Some(path) = relative {
          write!(f, " {path}")?;
        }

        Ok(())
      }
      Item::Recipe(recipe) => write!(f, "{}", recipe.color_display(Color::never())),
      Item::Set(set) => write!(f, "{set}"),
    }
  }
}
