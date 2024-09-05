use super::*;

#[derive(EnumDiscriminants, PartialEq, Debug, Clone, Serialize, Eq, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[strum_discriminants(name(AttributeDiscriminant))]
#[strum_discriminants(derive(EnumString, PartialOrd, Ord))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub(crate) enum Attribute<'src> {
  Alias(Name<'src>),
  Confirm(Option<StringLiteral<'src>>),
  Doc(Option<StringLiteral<'src>>),
  Extension(StringLiteral<'src>),
  Group(StringLiteral<'src>),
  Linux,
  Macos,
  NoCd,
  NoExitMessage,
  NoQuiet,
  PositionalArguments,
  Private,
  Script(Option<Interpreter<'src>>),
  Unix,
  Windows,
}

impl AttributeDiscriminant {
  fn argument_range(self) -> RangeInclusive<usize> {
    match self {
      Self::Confirm | Self::Doc => 0..=1,
      Self::Group | Self::Extension | Self::Alias => 1..=1,
      Self::Linux
      | Self::Macos
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::PositionalArguments
      | Self::Private
      | Self::Unix
      | Self::Windows => 0..=0,
      Self::Script => 0..=usize::MAX,
    }
  }
}

impl<'src> Attribute<'src> {
  pub(crate) fn new(
    name: Name<'src>,
    arguments: Vec<AttributeArgument<'src>>,
  ) -> CompileResult<'src, Self> {
    let discriminant = name
      .lexeme()
      .parse::<AttributeDiscriminant>()
      .ok()
      .ok_or_else(|| {
        name.error(CompileErrorKind::UnknownAttribute {
          attribute: name.lexeme(),
        })
      })?;

    let found = arguments.len();
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
      AttributeDiscriminant::Alias => Self::Alias(Self::extract_argument(name, arguments)?),
      AttributeDiscriminant::Confirm => {
        Self::Confirm(Self::extract_optional_argument(name, arguments)?)
      }
      AttributeDiscriminant::Doc => Self::Doc(Self::extract_optional_argument(name, arguments)?),
      AttributeDiscriminant::Extension => Self::Extension(Self::extract_argument(name, arguments)?),
      AttributeDiscriminant::Group => Self::Group(Self::extract_argument(name, arguments)?),
      AttributeDiscriminant::Linux => Self::Linux,
      AttributeDiscriminant::Macos => Self::Macos,
      AttributeDiscriminant::NoCd => Self::NoCd,
      AttributeDiscriminant::NoExitMessage => Self::NoExitMessage,
      AttributeDiscriminant::NoQuiet => Self::NoQuiet,
      AttributeDiscriminant::PositionalArguments => Self::PositionalArguments,
      AttributeDiscriminant::Private => Self::Private,
      AttributeDiscriminant::Script => Self::Script({
        let mut args_iter = arguments.into_iter();

        if let Some(value) = args_iter.next() {
          let command = value.try_into().map_err(|e| {
            CompileError::new(name.token, CompileErrorKind::Internal { message: e })
          })?;

          let mut arguments = Vec::new();
          for arg in args_iter {
            arguments.push(arg.try_into().map_err(|e| {
              CompileError::new(name.token, CompileErrorKind::Internal { message: e })
            })?);
          }

          Some(Interpreter { arguments, command })
        } else {
          None
        }
      }),
      AttributeDiscriminant::Unix => Self::Unix,
      AttributeDiscriminant::Windows => Self::Windows,
    })
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  fn extract_argument<T: TryFrom<AttributeArgument<'src>, Error = String>>(
    attribute: Name<'src>,
    arguments: Vec<AttributeArgument<'src>>,
  ) -> Result<T, CompileError<'src>> {
    arguments
      .into_iter()
      .next()
      .unwrap()
      .try_into()
      .map_err(|e| CompileError::new(attribute.token, CompileErrorKind::Internal { message: e }))
  }

  fn extract_optional_argument<T: TryFrom<AttributeArgument<'src>, Error = String>>(
    attribute: Name<'src>,
    arguments: Vec<AttributeArgument<'src>>,
  ) -> Result<Option<T>, CompileError<'src>> {
    let value = if let Some(value) = arguments.into_iter().next() {
      Some(value.try_into().map_err(|e| {
        CompileError::new(attribute.token, CompileErrorKind::Internal { message: e })
      })?)
    } else {
      None
    };
    Ok(value)
  }
}

impl<'src> Display for Attribute<'src> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())?;

    match self {
      Self::Alias(argument) => write!(f, "({argument})")?,
      Self::Confirm(Some(argument))
      | Self::Doc(Some(argument))
      | Self::Extension(argument)
      | Self::Group(argument) => write!(f, "({argument})")?,
      Self::Script(Some(shell)) => write!(f, "({shell})")?,
      Self::Confirm(None)
      | Self::Doc(None)
      | Self::Linux
      | Self::Macos
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::PositionalArguments
      | Self::Private
      | Self::Script(None)
      | Self::Unix
      | Self::Windows => {}
    }

    Ok(())
  }
}

impl<'src> PartialOrd for Attribute<'src> {
  fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<'src> cmp::Ord for Attribute<'src> {
  #[inline]
  fn cmp(&self, other: &Attribute<'src>) -> cmp::Ordering {
    use cmp::Ordering;
    use Attribute::*;

    let self_discr: AttributeDiscriminant = self.into();
    let other_discr: AttributeDiscriminant = other.into();
    match Ord::cmp(&self_discr, &other_discr) {
      Ordering::Equal => match (self, other) {
        (Alias(a), Alias(b)) => Ord::cmp(a.lexeme(), b.lexeme()),
        (Confirm(a), Confirm(b)) | (Doc(a), Doc(b)) => Ord::cmp(a, b),
        (Extension(a), Extension(b)) | (Group(a), Group(b)) => Ord::cmp(a, b),
        (Script(a), Script(b)) => Ord::cmp(a, b),
        _ => Ordering::Equal,
      },
      cmp => cmp,
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
}
