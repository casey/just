use super::*;

#[derive(EnumString, PartialEq, Debug, Clone, Serialize, Ord, PartialOrd, Eq, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Attribute<'src> {
  Confirm(Option<&'src str>),
  Group { name: &'src str },
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
    maybe_argument: Option<&'src str>,
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
    self.into()
  }
}

impl<'src> Display for Attribute<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    let attr_name = self.name();
    match self {
      Self::Confirm(Some(prompt)) => write!(f, "{attr_name}('{prompt}')"),
      Self::Group { name } => {
        let use_quotes = name.contains(char::is_whitespace);
        let mq = if use_quotes { "\"" } else { "" };
        write!(f, "{attr_name}({mq}{name}{mq})")
      }
      _other => write!(f, "{attr_name}"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn name() {
    assert_eq!(Attribute::NoExitMessage.name(), "no-exit-message");
  }

  #[test]
  fn group() {
    assert_eq!(
      Attribute::Group { name: "linter" }.to_string(),
      "group(linter)"
    );
  }
}
