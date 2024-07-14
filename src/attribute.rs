use super::*;

#[derive(
  EnumDiscriminants, PartialEq, Debug, Clone, Serialize, Ord, PartialOrd, Eq, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[strum_discriminants(name(AttributeDiscriminant))]
#[strum_discriminants(derive(EnumString))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub(crate) enum Attribute<'src> {
  Confirm(Option<StringLiteral<'src>>),
  Doc(Option<StringLiteral<'src>>),
  Group(StringLiteral<'src>),
  Linux,
  Macos,
  NoCd,
  NoExitMessage,
  NoQuiet,
  PositionalArguments,
  Private,
  Unix,
  Windows,
}

impl AttributeDiscriminant {
  fn argument_range(self) -> RangeInclusive<usize> {
    match self {
      Self::Confirm | Self::Doc => 0..=1,
      Self::Group => 1..=1,
      Self::Linux
      | Self::Macos
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::PositionalArguments
      | Self::Private
      | Self::Unix
      | Self::Windows => 0..=0,
    }
  }
}

impl<'src> Attribute<'src> {
  pub(crate) fn new(
    name: Name<'src>,
    arguments: Option<Vec<StringLiteral<'src>>>,
  ) -> CompileResult<'src, Self> {
    use AttributeDiscriminant::*;

    let discriminant = name
      .lexeme()
      .parse::<AttributeDiscriminant>()
      .ok()
      .ok_or_else(|| {
        name.error(CompileErrorKind::UnknownAttribute {
          attribute: name.lexeme(),
        })
      })?;

    let found = arguments.as_ref().map_or(0, |vec| vec.len());
    let range = discriminant.argument_range();
    if !range.contains(&found) {
      return Err(
        name.error(CompileErrorKind::AttributeArgumentCountMismatch {
          attribute: name.lexeme(),
          found,
          min: *range.start(),
          max: *range.end(),
        }),
      );
    }

    Ok(match discriminant {
      Confirm => Self::Confirm(arguments.and_then(|vec| vec.into_iter().next())),
      Doc => Self::Doc(arguments.and_then(|vec| vec.into_iter().next())),
      Group => Self::Group(arguments.and_then(|vec| vec.into_iter().next()).unwrap()),
      Linux => Self::Linux,
      Macos => Self::Macos,
      NoCd => Self::NoCd,
      NoExitMessage => Self::NoExitMessage,
      NoQuiet => Self::NoQuiet,
      PositionalArguments => Self::PositionalArguments,
      Private => Self::Private,
      Unix => Self::Unix,
      Windows => Self::Windows,
    })
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  fn arguments(&self) -> Option<Vec<&StringLiteral>> {
    match self {
      Self::Confirm(prompt) => prompt.as_ref().map(|arg| vec![arg]),
      Self::Doc(doc) => doc.as_ref().map(|arg| vec![arg]),
      Self::Group(group) => Some(vec![group]),
      Self::Linux
      | Self::Macos
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::PositionalArguments
      | Self::Private
      | Self::Unix
      | Self::Windows => None,
    }
  }
}

impl<'src> Display for Attribute<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())?;
    if let Some(arguments) = self.arguments() {
      write!(f, "(")?;
      for argument in arguments.iter().take(arguments.len() - 1) {
        write!(f, "{argument}, ")?;
      }
      if let Some(argument) = arguments.last() {
        write!(f, "{argument}")?;
      }
      write!(f, ")")?;
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
