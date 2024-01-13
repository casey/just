use super::*;

#[derive(EnumString, PartialEq, Debug, Clone, Serialize, Ord, PartialOrd, Eq, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Attribute<'src> {
  Confirm(Option<String>),
  Group { name: String },
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

  pub(crate) fn parse(name: &Name, maybe_argument: Option<String>) -> CompileResult<'src, Self> {
    let name_str = name.lexeme();
    Ok(match (name_str, maybe_argument) {
      ("group", Some(name)) => Self::Group { name},
      ("confirm", Some(argument)) => Self::Comfirm

      _ => todo!()
    })
    /*
    match (name, maybe_argument) {
      ("group", Some(name)) => Ok(Attribute::Group { name }),
      ("group", None) => Err(CompileErrorKind::InvalidAttributeArgument {
        name: name.to_string(),
        expected: true,
      }),
      (other, None) => other
        .parse()
        .map_err(|_| CompileErrorKind::UnknownAttribute { attribute: name }),
      (_other, Some(_)) => Err(CompileErrorKind::InvalidAttributeArgument {
        name: name.to_string(),
        expected: false,
      }),
    }
    */

    todo!()
  }


  pub(crate) fn from_name(name: Name) -> Option<Self> {
    name.lexeme().parse().ok()
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn with_argument(
    self,
    name: Name<'src>,
    argument: StringLiteral<'src>,
  ) -> CompileResult<'src, Self> {
    match self {
      Self::Confirm(_) => Ok(Self::Confirm(Some(argument))),
      _ => Err(name.error(CompileErrorKind::UnexpectedAttributeArgument { attribute: self })),
    }
  }
}

impl<'src> Display for Attribute<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    let attr_name = self.name();
    match self {
      Self::Confirm(Some(prompt)) => write!(f, "{attr_name}({prompt})"),
      Self::Group { name } => write!(f, "{attr_name}({name})"),
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
      Attribute::Group {
        name: "linter".to_string()
      }
      .to_string(),
      "group(linter)"
    );
  }
}
