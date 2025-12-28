use super::*;

#[allow(clippy::large_enum_variant)]
#[derive(
  EnumDiscriminants, PartialEq, Debug, Clone, Serialize, Ord, PartialOrd, Eq, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[strum_discriminants(name(AttributeDiscriminant))]
#[strum_discriminants(derive(EnumString, Ord, PartialOrd))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub(crate) enum Attribute<'src> {
  Arg {
    long: Option<StringLiteral<'src>>,
    #[serde(skip)]
    long_token: Option<Token<'src>>,
    name: StringLiteral<'src>,
    #[serde(skip)]
    name_token: Token<'src>,
    #[serde(skip)]
    pattern: Option<Pattern>,
    #[serde(rename = "pattern")]
    pattern_literal: Option<StringLiteral<'src>>,
    short: Option<StringLiteral<'src>>,
    #[serde(skip)]
    short_token: Option<Token<'src>>,
  },
  Confirm(Option<StringLiteral<'src>>),
  Default,
  Doc(Option<StringLiteral<'src>>),
  ExitMessage,
  Extension(StringLiteral<'src>),
  Group(StringLiteral<'src>),
  Linux,
  Macos,
  Metadata(Vec<StringLiteral<'src>>),
  NoCd,
  NoExitMessage,
  NoQuiet,
  Openbsd,
  Parallel,
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
      Self::Default
      | Self::ExitMessage
      | Self::Linux
      | Self::Macos
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::Openbsd
      | Self::Parallel
      | Self::PositionalArguments
      | Self::Private
      | Self::Unix
      | Self::Windows => 0..=0,
      Self::Confirm | Self::Doc => 0..=1,
      Self::Script => 0..=usize::MAX,
      Self::Arg | Self::Extension | Self::Group | Self::WorkingDirectory => 1..=1,
      Self::Metadata => 1..=usize::MAX,
    }
  }
}

impl<'src> Attribute<'src> {
  pub(crate) fn new(
    name: Name<'src>,
    arguments: Vec<(Token<'src>, StringLiteral<'src>)>,
    mut keyword_arguments: BTreeMap<&'src str, (Name<'src>, Token<'src>, StringLiteral<'src>)>,
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

    let (tokens, arguments): (Vec<Token>, Vec<StringLiteral>) = arguments.into_iter().unzip();

    let attribute = match discriminant {
      AttributeDiscriminant::Arg => {
        let name = arguments.into_iter().next().unwrap();
        let name_token = tokens.into_iter().next().unwrap();

        let (long_token, long) =
          if let Some((_name, token, literal)) = keyword_arguments.remove("long") {
            if literal.cooked.contains('=') {
              return Err(token.error(CompileErrorKind::OptionNameContainsEqualSign {
                parameter: name.cooked,
              }));
            }

            if literal.cooked.is_empty() {
              return Err(token.error(CompileErrorKind::OptionNameEmpty {
                parameter: name.cooked,
              }));
            }

            (Some(token), Some(literal))
          } else {
            (None, None)
          };

        let (short_token, short) =
          if let Some((_name, token, literal)) = keyword_arguments.remove("short") {
            if literal.cooked.contains('=') {
              return Err(token.error(CompileErrorKind::OptionNameContainsEqualSign {
                parameter: name.cooked,
              }));
            }

            if literal.cooked.is_empty() {
              return Err(token.error(CompileErrorKind::OptionNameEmpty {
                parameter: name.cooked,
              }));
            }

            if literal.cooked.chars().count() != 1 {
              return Err(
                token.error(CompileErrorKind::ShortOptionWithMultipleCharacters {
                  parameter: name.cooked,
                }),
              );
            }

            (Some(token), Some(literal))
          } else {
            (None, None)
          };

        let (pattern_literal, pattern) = keyword_arguments
          .remove("pattern")
          .map(|(_name, token, literal)| {
            let pattern = Pattern::new(token, &literal)?;
            Ok((Some(literal), Some(pattern)))
          })
          .transpose()?
          .unwrap_or((None, None));

        Self::Arg {
          long,
          long_token,
          name,
          name_token,
          pattern,
          pattern_literal,
          short,
          short_token,
        }
      }
      AttributeDiscriminant::Confirm => Self::Confirm(arguments.into_iter().next()),
      AttributeDiscriminant::Default => Self::Default,
      AttributeDiscriminant::Doc => Self::Doc(arguments.into_iter().next()),
      AttributeDiscriminant::ExitMessage => Self::ExitMessage,
      AttributeDiscriminant::Extension => Self::Extension(arguments.into_iter().next().unwrap()),
      AttributeDiscriminant::Group => Self::Group(arguments.into_iter().next().unwrap()),
      AttributeDiscriminant::Linux => Self::Linux,
      AttributeDiscriminant::Macos => Self::Macos,
      AttributeDiscriminant::Metadata => Self::Metadata(arguments),
      AttributeDiscriminant::NoCd => Self::NoCd,
      AttributeDiscriminant::NoExitMessage => Self::NoExitMessage,
      AttributeDiscriminant::NoQuiet => Self::NoQuiet,
      AttributeDiscriminant::Openbsd => Self::Openbsd,
      AttributeDiscriminant::Parallel => Self::Parallel,
      AttributeDiscriminant::PositionalArguments => Self::PositionalArguments,
      AttributeDiscriminant::Private => Self::Private,
      AttributeDiscriminant::Script => Self::Script({
        let mut arguments = arguments.into_iter();
        arguments.next().map(|command| Interpreter {
          command,
          arguments: arguments.collect(),
        })
      }),
      AttributeDiscriminant::Unix => Self::Unix,
      AttributeDiscriminant::Windows => Self::Windows,
      AttributeDiscriminant::WorkingDirectory => {
        Self::WorkingDirectory(arguments.into_iter().next().unwrap())
      }
    };

    if let Some((_name, (keyword_name, _token, _literal))) = keyword_arguments.into_iter().next() {
      return Err(
        keyword_name.error(CompileErrorKind::UnknownAttributeKeyword {
          attribute: name.lexeme(),
          keyword: keyword_name.lexeme(),
        }),
      );
    }

    Ok(attribute)
  }

  pub(crate) fn discriminant(&self) -> AttributeDiscriminant {
    self.into()
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  pub(crate) fn repeatable(&self) -> bool {
    matches!(
      self,
      Attribute::Arg { .. } | Attribute::Group(_) | Attribute::Metadata(_),
    )
  }
}

impl Display for Attribute<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())?;

    match self {
      Self::Arg {
        long,
        long_token: _,
        name,
        name_token: _,
        pattern: _,
        pattern_literal,
        short,
        short_token: _,
      } => {
        write!(f, "({name}")?;

        if let Some(long) = long {
          write!(f, ", long={long}")?;
        }

        if let Some(short) = short {
          write!(f, ", short={short}")?;
        }

        if let Some(pattern) = pattern_literal {
          write!(f, ", pattern={pattern}")?;
        }

        write!(f, ")")?;
      }
      Self::Confirm(None)
      | Self::Default
      | Self::Doc(None)
      | Self::ExitMessage
      | Self::Linux
      | Self::Macos
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::Openbsd
      | Self::Parallel
      | Self::PositionalArguments
      | Self::Private
      | Self::Script(None)
      | Self::Unix
      | Self::Windows => {}
      Self::Confirm(Some(argument))
      | Self::Doc(Some(argument))
      | Self::Extension(argument)
      | Self::Group(argument)
      | Self::WorkingDirectory(argument) => write!(f, "({argument})")?,
      Self::Metadata(arguments) => {
        write!(f, "(")?;
        for (i, argument) in arguments.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{argument}")?;
        }
        write!(f, ")")?;
      }
      Self::Script(Some(shell)) => write!(f, "({shell})")?,
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
