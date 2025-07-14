use super::*;

/// A single top-level item
#[derive(Debug, Clone)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src, Namepath<'src>>),
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
    doc: Option<String>,
    groups: Vec<String>,
    name: Name<'src>,
    optional: bool,
    relative: Option<StringLiteral<'src>>,
  },
  Recipe(UnresolvedRecipe<'src>),
  Set(Set<'src>),
  Unexport {
    name: Name<'src>,
  },
}

impl Display for Item<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Alias(alias) => write!(f, "{alias}"),
      Self::Assignment(assignment) => write!(f, "{assignment}"),
      Self::Comment(comment) => write!(f, "{comment}"),
      Self::Import {
        relative, optional, ..
      } => {
        write!(f, "import")?;

        if *optional {
          write!(f, "?")?;
        }

        write!(f, " {relative}")
      }
      Self::Module {
        doc,
        groups,
        name,
        relative,
        optional,
        ..
      } => {
        if let Some(docstr) = doc {
          writeln!(f, "# {docstr}")?;
        }

        for group in groups.iter() {
          writeln!(f, "[group('{}')]", group)?;
        }

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
      Self::Recipe(recipe) => write!(f, "{}", recipe.color_display(Color::never())),
      Self::Set(set) => write!(f, "{set}"),
      Self::Unexport { name } => write!(f, "unexport {name}"),
    }
  }
}
