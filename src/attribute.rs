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
    help: Option<StringLiteral<'src>>,
    long: Option<StringLiteral<'src>>,
    #[serde(skip)]
    long_key: Option<Token<'src>>,
    name: StringLiteral<'src>,
    pattern: Option<Pattern<'src>>,
    short: Option<StringLiteral<'src>>,
    value: Option<StringLiteral<'src>>,
  },
  Confirm(Option<StringLiteral<'src>>),
  Default,
  Doc(Option<StringLiteral<'src>>),
  Env(StringLiteral<'src>, StringLiteral<'src>),
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
  Script(Option<Interpreter<StringLiteral<'src>>>),
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
      Self::Env => 2..=2,
      Self::Metadata => 1..=usize::MAX,
    }
  }
}

impl<'src> Attribute<'src> {
  fn check_option_name(
    parameter: &StringLiteral<'src>,
    literal: &StringLiteral<'src>,
  ) -> CompileResult<'src> {
    if literal.cooked.contains('=') {
      return Err(
        literal
          .token
          .error(CompileErrorKind::OptionNameContainsEqualSign {
            parameter: parameter.cooked.clone(),
          }),
      );
    }

    if literal.cooked.is_empty() {
      return Err(literal.token.error(CompileErrorKind::OptionNameEmpty {
        parameter: parameter.cooked.clone(),
      }));
    }

    Ok(())
  }

  pub(crate) fn new(
    name: Name<'src>,
    arguments: Vec<StringLiteral<'src>>,
    mut keyword_arguments: BTreeMap<&'src str, (Name<'src>, Option<StringLiteral<'src>>)>,
  ) -> CompileResult<'src, Self> {
    let discriminant = name
      .lexeme()
      .parse::<AttributeDiscriminant>()
      .map_err(|_| {
        name.error(CompileErrorKind::UnknownAttribute {
          attribute: name.lexeme(),
        })
      })?;

    let found = arguments.len();
    let range = discriminant.argument_range();
    if !range.contains(&found) {
      return Err(
        name.error(CompileErrorKind::AttributeArgumentCountMismatch {
          attribute: name,
          found,
          min: *range.start(),
          max: *range.end(),
        }),
      );
    }

    let attribute = match discriminant {
      AttributeDiscriminant::Arg => {
        let arg = arguments.into_iter().next().unwrap();

        let (long, long_key) = keyword_arguments
          .remove("long")
          .map(|(name, literal)| {
            if let Some(literal) = literal {
              Self::check_option_name(&arg, &literal)?;
              Ok((Some(literal), None))
            } else {
              Ok((Some(arg.clone()), Some(*name)))
            }
          })
          .transpose()?
          .unwrap_or((None, None));

        let short = Self::remove_required(&mut keyword_arguments, "short")?
          .map(|(_key, literal)| {
            Self::check_option_name(&arg, &literal)?;

            if literal.cooked.chars().count() != 1 {
              return Err(literal.token.error(
                CompileErrorKind::ShortOptionWithMultipleCharacters {
                  parameter: arg.cooked.clone(),
                },
              ));
            }

            Ok(literal)
          })
          .transpose()?;

        let pattern = Self::remove_required(&mut keyword_arguments, "pattern")?
          .map(|(_key, literal)| Pattern::new(&literal))
          .transpose()?;

        let value = Self::remove_required(&mut keyword_arguments, "value")?
          .map(|(key, literal)| {
            if long.is_none() && short.is_none() {
              return Err(key.error(CompileErrorKind::ArgAttributeValueRequiresOption));
            }
            Ok(literal)
          })
          .transpose()?;

        let help =
          Self::remove_required(&mut keyword_arguments, "help")?.map(|(_key, literal)| literal);

        Self::Arg {
          help,
          long,
          long_key,
          name: arg,
          pattern,
          short,
          value,
        }
      }
      AttributeDiscriminant::Confirm => Self::Confirm(arguments.into_iter().next()),
      AttributeDiscriminant::Default => Self::Default,
      AttributeDiscriminant::Doc => Self::Doc(arguments.into_iter().next()),
      AttributeDiscriminant::Env => {
        let [key, value]: [StringLiteral; 2] = arguments.try_into().unwrap();
        Self::Env(key, value)
      }
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

    if let Some((_name, (keyword_name, _literal))) = keyword_arguments.into_iter().next() {
      return Err(
        keyword_name.error(CompileErrorKind::UnknownAttributeKeyword {
          attribute: name.lexeme(),
          keyword: keyword_name.lexeme(),
        }),
      );
    }

    Ok(attribute)
  }

  fn remove_required(
    keyword_arguments: &mut BTreeMap<&'src str, (Name<'src>, Option<StringLiteral<'src>>)>,
    key: &'src str,
  ) -> CompileResult<'src, Option<(Name<'src>, StringLiteral<'src>)>> {
    let Some((key, literal)) = keyword_arguments.remove(key) else {
      return Ok(None);
    };

    let literal =
      literal.ok_or_else(|| key.error(CompileErrorKind::AttributeKeyMissingValue { key }))?;

    Ok(Some((key, literal)))
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
      Attribute::Arg { .. } | Attribute::Env(_, _) | Attribute::Group(_) | Attribute::Metadata(_),
    )
  }
}

impl Display for Attribute<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())?;

    match self {
      Self::Arg {
        help,
        long,
        long_key: _,
        name,
        pattern,
        short,
        value,
      } => {
        write!(f, "({name}")?;

        if let Some(long) = long {
          write!(f, ", long={long}")?;
        }

        if let Some(short) = short {
          write!(f, ", short={short}")?;
        }

        if let Some(pattern) = pattern {
          write!(f, ", pattern={}", pattern.token.lexeme())?;
        }

        if let Some(value) = value {
          write!(f, ", value={value}")?;
        }

        if let Some(help) = help {
          write!(f, ", help={help}")?;
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
      Self::Env(key, value) => write!(f, "({key}, {value})")?,
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
