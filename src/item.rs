use super::*;

/// A single top-level item
#[derive(Debug, Clone)]
pub(crate) enum Item<'src> {
  Alias(Alias<'src, Namepath<'src>>),
  Assignment(Assignment<'src>),
  Comment(&'src str),
  Function(FunctionDefinition<'src>),
  Import {
    absolute: Option<PathBuf>,
    attributes: AttributeSet<'src>,
    optional: bool,
    relative: StringLiteral<'src>,
  },
  Module {
    absolute: Option<PathBuf>,
    attributes: AttributeSet<'src>,
    doc: Option<String>,
    name: Name<'src>,
    optional: bool,
    relative: Option<StringLiteral<'src>>,
  },
  Newline,
  Recipe(UnresolvedRecipe<'src>),
  Set(Set<'src>),
  Unexport {
    attributes: AttributeSet<'src>,
    name: Name<'src>,
  },
}

impl<'src> Item<'src> {
  fn attributes(&self) -> Option<&AttributeSet<'src>> {
    match self {
      Self::Alias(alias) => Some(&alias.attributes),
      Self::Assignment(assignment) => Some(&assignment.attributes),
      Self::Comment(_) | Self::Newline => None,
      Self::Function(function) => Some(&function.attributes),
      Self::Import { attributes, .. }
      | Self::Module { attributes, .. }
      | Self::Unexport { attributes, .. } => Some(attributes),
      Self::Recipe(recipe) => Some(&recipe.attributes),
      Self::Set(set) => Some(&set.attributes),
    }
  }

  pub(crate) fn enabled(&self) -> bool {
    self.attributes().is_none_or(AttributeSet::is_enabled)
  }

  fn doc_comment(&self) -> Option<&str> {
    match self {
      Self::Module {
        attributes, doc, ..
      } => {
        if attributes.contains(AttributeKind::Doc) {
          None
        } else {
          doc.as_deref()
        }
      }
      Self::Recipe(recipe) => {
        if recipe.attributes.contains(AttributeKind::Doc) {
          None
        } else {
          recipe.doc.as_deref()
        }
      }
      _ => None,
    }
  }
}

impl ColorDisplay for Item<'_> {
  fn fmt(&self, f: &mut Formatter, color: Color) -> fmt::Result {
    if let Some(doc) = self.doc_comment() {
      writeln!(f, "# {doc}")?;
    }

    if let Some(attributes) = self.attributes() {
      for attribute in attributes {
        writeln!(f, "[{attribute}]")?;
      }
    }

    match self {
      Self::Alias(alias) => write!(f, "{alias}"),
      Self::Assignment(assignment) => write!(f, "{assignment}"),
      Self::Comment(comment) => write!(f, "{comment}"),
      Self::Function(function) => {
        write!(f, "{}(", function.name)?;
        for (i, (parameter, _number)) in function.parameters.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{parameter}")?;
        }
        write!(f, ") := {}", function.body)
      }
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
        name,
        optional,
        relative,
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
      Self::Newline => Ok(()),
      Self::Recipe(recipe) => write!(f, "{}", recipe.color_display(color)),
      Self::Set(set) => write!(f, "{set}"),
      Self::Unexport { name, .. } => write!(f, "unexport {name}"),
    }
  }
}
