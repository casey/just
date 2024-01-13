use super::*;

#[derive(EnumString, PartialEq, Debug, Clone, Serialize, Ord, PartialOrd, Eq, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Attribute<'src> {
  Confirm(Option<StringLiteral<'src>>),
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
  pub(crate) fn from_name(name: Name) -> Option<Self> {
    name.lexeme().parse().ok()
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn with_argument(
    self,
    argument: StringLiteral<'src>,
  ) -> Result<Self, CompileErrorKind<'src>> {
    match self {
      Self::Confirm(_) => Ok(Self::Confirm(Some(argument))),
      // Return error for all attributes that don't accept arguments
      _ => Err(CompileErrorKind::UnexpectedAttributeArgument { attribute: self }),
    }
  }

  fn argument(&self) -> Option<&StringLiteral> {
    if let Self::Confirm(prompt) = self {
      prompt.as_ref()
    } else {
      None
    }
  }
}

impl<'src> Display for Attribute<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", self.name())?;

    if let Some(argument) = self.argument() {
      write!(f, "({})", argument)?;
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
