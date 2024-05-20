use super::*;

#[derive(EnumString, PartialEq, Debug, Clone, Serialize, Ord, PartialOrd, Eq, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Attribute<'src> {
  Confirm(Option<StringLiteral<'src>>),
  #[strum(disabled)]
  Group {
    name: StringLiteral<'src>,
  },
  Linux,
  Macos,
  NoCd,
  NoExitMessage,
  Private,
  NoQuiet,
  Unix,
  Windows,
}

impl<'src> Attribute<'src> {
  pub(crate) fn parse(
    name: Name<'src>,
    maybe_argument: Option<StringLiteral<'src>>,
  ) -> CompileResult<'src, Self> {
    let name_str = name.lexeme();

    Ok(match (name_str, maybe_argument) {
      ("group", Some(name)) => Self::Group { name },
      ("group", None) => {
        return Err(name.error(CompileErrorKind::MissingAttributeArgument {
          attribute_name: "group".into(),
        }))
      }
      ("confirm", argument) => Self::Confirm(argument),
      (other_attribute, None) => other_attribute.parse().map_err(|_| {
        name.error(CompileErrorKind::UnknownAttribute {
          attribute: name_str,
        })
      })?,
      (_other_attribute, Some(_)) => {
        return Err(name.error(CompileErrorKind::UnexpectedAttributeArgument {
          attribute: name_str,
        }))
      }
    })
  }

  pub(crate) fn name(&self) -> &'static str {
    // Necessary because the Group variant is disabled for EnumString
    if let Self::Group { .. } = self {
      "group"
    } else {
      self.into()
    }
  }

  fn argument(&self) -> Option<&StringLiteral> {
    match self {
      Self::Confirm(prompt) => prompt.as_ref(),
      Self::Group { name } => Some(name),
      _ => None,
    }
  }
}

impl<'src> Display for Attribute<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self.name())?;

    if let Some(argument) = self.argument() {
      write!(f, "({argument})")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn name() {
    assert_eq!(Attribute::NoExitMessage.name(), "no-exit-message");
  }
}
