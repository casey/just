use super::*;

pub(crate) type EvaluatedAttribute<'src> = Attribute<'src, String>;

#[allow(clippy::large_enum_variant)]
#[derive(
  EnumDiscriminants, PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Serialize, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[strum_discriminants(name(AttributeDiscriminant))]
#[strum_discriminants(derive(EnumString, Ord, PartialOrd))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub(crate) enum Attribute<'src, T = Expression<'src>> {
  Android,
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
  Confirm(Option<Expression<'src>>),
  Default,
  Doc(Option<StringLiteral<'src>>),
  Dragonfly,
  Env(StringLiteral<'src>, StringLiteral<'src>),
  ExitMessage,
  Extension(StringLiteral<'src>),
  Freebsd,
  Group(StringLiteral<'src>),
  Linux,
  Macos,
  Metadata(Vec<StringLiteral<'src>>),
  Netbsd,
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
  WorkingDirectory(T),
}

impl AttributeDiscriminant {
  pub(crate) fn accepts_keyword_arguments(self) -> bool {
    matches!(self, Self::Arg)
  }

  fn argument_range(self) -> RangeInclusive<usize> {
    use AttributeDiscriminant::*;

    match self {
      Android | Default | Dragonfly | ExitMessage | Freebsd | Linux | Macos | Netbsd | NoCd
      | NoExitMessage | NoQuiet | Openbsd | Parallel | PositionalArguments | Private | Unix
      | Windows => 0..=0,
      Confirm | Doc => 0..=1,
      Script => 0..=usize::MAX,
      Arg | Extension | Group | WorkingDirectory => 1..=1,
      Env => 2..=2,
      Metadata => 1..=usize::MAX,
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
    discriminant: AttributeDiscriminant,
    arguments: Vec<(Token<'src>, Expression<'src>)>,
    mut keyword_arguments: BTreeMap<&'src str, (Name<'src>, Option<StringLiteral<'src>>)>,
  ) -> CompileResult<'src, Self> {
    use AttributeDiscriminant::*;

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

    if let Confirm | WorkingDirectory = discriminant {
      if let Some((_name, (keyword, _literal))) = keyword_arguments.into_iter().next() {
        return Err(keyword.error(CompileErrorKind::UnknownAttributeKeyword {
          attribute: name.lexeme(),
          keyword: keyword.lexeme(),
        }));
      }

      let argument = arguments
        .into_iter()
        .next()
        .map(|(_token, expression)| expression);

      return Ok(match discriminant {
        Confirm => Self::Confirm(argument),
        WorkingDirectory => Self::WorkingDirectory(argument.unwrap()),
        _ => unreachable!(),
      });
    }

    let arguments = arguments
      .into_iter()
      .map(|(token, argument)| {
        let Expression::StringLiteral { string_literal } = argument else {
          return Err(token.error(CompileErrorKind::AttributeArgumentExpression {
            attribute: name.lexeme(),
          }));
        };
        Ok(string_literal)
      })
      .collect::<CompileResult<Vec<StringLiteral>>>()?;

    let attribute = match discriminant {
      Arg => {
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
      Android => Self::Android,
      Confirm | WorkingDirectory => unreachable!(),
      Default => Self::Default,
      Doc => Self::Doc(arguments.into_iter().next()),
      Dragonfly => Self::Dragonfly,
      Env => {
        let [key, value]: [StringLiteral; 2] = arguments.try_into().unwrap();
        Self::Env(key, value)
      }
      ExitMessage => Self::ExitMessage,
      Extension => Self::Extension(arguments.into_iter().next().unwrap()),
      Freebsd => Self::Freebsd,
      Group => Self::Group(arguments.into_iter().next().unwrap()),
      Linux => Self::Linux,
      Macos => Self::Macos,
      Metadata => Self::Metadata(arguments),
      Netbsd => Self::Netbsd,
      NoCd => Self::NoCd,
      NoExitMessage => Self::NoExitMessage,
      NoQuiet => Self::NoQuiet,
      Openbsd => Self::Openbsd,
      Parallel => Self::Parallel,
      PositionalArguments => Self::PositionalArguments,
      Private => Self::Private,
      Script => Self::Script({
        let mut arguments = arguments.into_iter();
        arguments.next().map(|command| Interpreter {
          command,
          arguments: arguments.collect(),
        })
      }),
      Unix => Self::Unix,
      Windows => Self::Windows,
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

  pub(crate) fn evaluate(
    self,
    assignments: &Table<'src, Assignment<'src>>,
    overrides: &HashMap<Number, String>,
  ) -> RunResult<'src, EvaluatedAttribute<'src>> {
    use Attribute::*;
    Ok(match self {
      Android => Android,
      Arg {
        help,
        long,
        long_key,
        name,
        pattern,
        short,
        value,
      } => Arg {
        help,
        long,
        long_key,
        name,
        pattern,
        short,
        value,
      },
      Confirm(argument) => Confirm(argument),
      Default => Default,
      Doc(argument) => Doc(argument),
      Dragonfly => Dragonfly,
      Env(key, value) => Env(key, value),
      ExitMessage => ExitMessage,
      Extension(argument) => Extension(argument),
      Freebsd => Freebsd,
      Group(argument) => Group(argument),
      Linux => Linux,
      Macos => Macos,
      Metadata(arguments) => Metadata(arguments),
      Netbsd => Netbsd,
      NoCd => NoCd,
      NoExitMessage => NoExitMessage,
      NoQuiet => NoQuiet,
      Openbsd => Openbsd,
      Parallel => Parallel,
      PositionalArguments => PositionalArguments,
      Private => Private,
      Script(argument) => Script(argument),
      Unix => Unix,
      Windows => Windows,
      WorkingDirectory(expression) => WorkingDirectory(Evaluator::evaluate_const_expression(
        assignments,
        overrides,
        &Scope::root(),
        &expression,
      )?),
    })
  }
}

impl<T> Attribute<'_, T> {
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

impl<T: Display> Display for Attribute<'_, T> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use Attribute::*;

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
      Android | Confirm(None) | Default | Doc(None) | Dragonfly | ExitMessage | Freebsd | Linux
      | Macos | Netbsd | NoCd | NoExitMessage | NoQuiet | Openbsd | Parallel
      | PositionalArguments | Private | Script(None) | Unix | Windows => {}
      Confirm(Some(argument)) => write!(f, "({argument})")?,
      WorkingDirectory(argument) => write!(f, "({argument})")?,
      Doc(Some(argument)) | Extension(argument) | Group(argument) => {
        write!(f, "({argument})")?;
      }
      Env(key, value) => write!(f, "({key}, {value})")?,
      Metadata(arguments) => {
        write!(f, "(")?;
        for (i, argument) in arguments.iter().enumerate() {
          if i > 0 {
            write!(f, ", ")?;
          }
          write!(f, "{argument}")?;
        }
        write!(f, ")")?;
      }
      Script(Some(shell)) => write!(f, "({shell})")?,
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn name() {
    assert_eq!(EvaluatedAttribute::NoExitMessage.name(), "no-exit-message");
  }
}
