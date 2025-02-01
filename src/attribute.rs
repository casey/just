use super::*;

#[derive(
  EnumDiscriminants, PartialEq, Debug, Clone, Serialize, Ord, PartialOrd, Eq, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[strum_discriminants(name(AttributeDiscriminant))]
#[strum_discriminants(derive(EnumString, Ord, PartialOrd))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub(crate) enum Attribute<'src> {
  Alias(Name<'src>, StringLiteral<'src>),
  Confirm(Option<StringLiteral<'src>>),
  Doc(Option<StringLiteral<'src>>),
  ExitMessage,
  Extension(StringLiteral<'src>),
  Group(StringLiteral<'src>),
  Linux,
  Macos,
  NoCd,
  NoExitMessage,
  NoQuiet,
  Openbsd,
  PositionalArguments,
  Private,
  Script(Option<Interpreter<'src>>),
  Unix,
  Windows,
  WorkingDirectory(StringLiteral<'src>),
}

impl AttributeDiscriminant {
  fn argument_range(self) -> RangeInclusive<usize> {
    match self {
      Self::Confirm | Self::Doc => 0..=1,
      Self::Alias | Self::Group | Self::Extension | Self::WorkingDirectory => 1..=1,
      Self::ExitMessage
      | Self::Linux
      | Self::Macos
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::Openbsd
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
    arguments: Vec<(Token<'src>, StringLiteral<'src>)>,
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

    let mut arguments = arguments.into_iter();
    let (token, argument) = arguments
      .next()
      .map(|(token, arg)| (Some(token), Some(arg)))
      .unwrap_or_default();

    Ok(match discriminant {
      AttributeDiscriminant::Alias => {
        let string_literal = argument.unwrap();
        let delim = string_literal.kind.delimiter_len();
        let token = token.unwrap();
        let token = Token {
          kind: TokenKind::Identifier,
          column: token.column + delim,
          length: token.length - (delim * 2),
          offset: token.offset + delim,
          ..token
        };

        let alias = token.lexeme();
        let valid_alias = alias.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');

        if alias.is_empty() || !valid_alias {
          return Err(token.error(CompileErrorKind::InvalidAliasName {
            name: token.lexeme(),
          }));
        }

        Self::Alias(Name::from_identifier(token), string_literal)
      }
      AttributeDiscriminant::Confirm => Self::Confirm(argument),
      AttributeDiscriminant::Doc => Self::Doc(argument),
      AttributeDiscriminant::ExitMessage => Self::ExitMessage,
      AttributeDiscriminant::Extension => Self::Extension(argument.unwrap()),
      AttributeDiscriminant::Group => Self::Group(argument.unwrap()),
      AttributeDiscriminant::Linux => Self::Linux,
      AttributeDiscriminant::Macos => Self::Macos,
      AttributeDiscriminant::NoCd => Self::NoCd,
      AttributeDiscriminant::NoExitMessage => Self::NoExitMessage,
      AttributeDiscriminant::NoQuiet => Self::NoQuiet,
      AttributeDiscriminant::Openbsd => Self::Openbsd,
      AttributeDiscriminant::PositionalArguments => Self::PositionalArguments,
      AttributeDiscriminant::Private => Self::Private,
      AttributeDiscriminant::Script => Self::Script({
        argument.map(|command| Interpreter {
          command,
          arguments: arguments.map(|(_, arg)| arg).collect(),
        })
      }),
      AttributeDiscriminant::Unix => Self::Unix,
      AttributeDiscriminant::Windows => Self::Windows,
      AttributeDiscriminant::WorkingDirectory => Self::WorkingDirectory(argument.unwrap()),
    })
  }

  pub(crate) fn discriminant(&self) -> AttributeDiscriminant {
    self.into()
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn repeatable(&self) -> bool {
    matches!(self, Attribute::Group(_) | Attribute::Alias(_, _))
  }
}

impl Display for Attribute<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())?;

    match self {
      Self::Alias(_, argument)
      | Self::Confirm(Some(argument))
      | Self::Doc(Some(argument))
      | Self::Extension(argument)
      | Self::Group(argument)
      | Self::WorkingDirectory(argument) => write!(f, "({argument})")?,
      Self::Script(Some(shell)) => write!(f, "({shell})")?,
      Self::Confirm(None)
      | Self::Doc(None)
      | Self::ExitMessage
      | Self::Linux
      | Self::Macos
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::Openbsd
      | Self::PositionalArguments
      | Self::Private
      | Self::Script(None)
      | Self::Unix
      | Self::Windows => {}
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
