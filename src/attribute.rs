use super::*;

#[allow(clippy::large_enum_variant)]
#[derive(
  Clone, Debug, EnumDiscriminants, Eq, IntoStaticStr, Ord, PartialEq, PartialOrd, Serialize,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[strum_discriminants(name(AttributeDiscriminant))]
#[strum_discriminants(derive(EnumString, Ord, PartialOrd))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub(crate) enum Attribute<'src> {
  Android,
  Arg {
    #[serde(skip)]
    flag: Option<Token<'src>>,
    help: Option<StringLiteral<'src>>,
    long: Option<StringLiteral<'src>>,
    #[serde(skip)]
    long_key: Option<Token<'src>>,
    name: StringLiteral<'src>,
    pattern: Option<Pattern<'src>>,
    short: Option<StringLiteral<'src>>,
    value: Option<Expression<'src>>,
  },
  Cache {
    inputs: Option<Expression<'src>>,
    outputs: Option<Expression<'src>>,
  },
  Confirm(Option<Expression<'src>>),
  Continue(BTreeSet<Signal>),
  Default,
  Doc(Option<StringLiteral<'src>>),
  Dragonfly,
  Env(Expression<'src>, Expression<'src>),
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
  Shell,
  Unix,
  Windows,
  WorkingDirectory(Expression<'src>),
}

impl AttributeDiscriminant {
  fn accepts_expressions(self) -> bool {
    matches!(self, Self::Confirm | Self::Env | Self::WorkingDirectory)
  }

  pub(crate) fn accepts_keyword_arguments(self) -> bool {
    matches!(self, Self::Arg | Self::Cache)
  }

  fn argument_range(self) -> RangeInclusive<usize> {
    match self {
      Self::Android
      | Self::Cache
      | Self::Default
      | Self::Dragonfly
      | Self::ExitMessage
      | Self::Freebsd
      | Self::Linux
      | Self::Macos
      | Self::Netbsd
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::Openbsd
      | Self::Parallel
      | Self::PositionalArguments
      | Self::Private
      | Self::Shell
      | Self::Unix
      | Self::Windows => 0..=0,
      Self::Confirm | Self::Doc => 0..=1,
      Self::Continue | Self::Script => 0..=usize::MAX,
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
    discriminant: AttributeDiscriminant,
    arguments: Vec<(Token<'src>, Expression<'src>)>,
    mut keyword_arguments: BTreeMap<&'src str, (Name<'src>, Option<Expression<'src>>)>,
  ) -> CompileResult<'src, Self> {
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

    if discriminant.accepts_expressions() {
      if let Some((_name, (key, _literal))) = keyword_arguments.pop_first() {
        return Err(key.error(CompileErrorKind::UnknownAttributeKey {
          attribute: name.lexeme(),
          key: key.lexeme(),
        }));
      }

      return match discriminant {
        AttributeDiscriminant::Confirm => Ok(Self::Confirm(
          arguments.into_iter().next().map(|(_, expr)| expr),
        )),
        AttributeDiscriminant::Env => {
          let mut arguments = arguments.into_iter();
          let (_, key) = arguments.next().unwrap();
          let (_, value) = arguments.next().unwrap();
          Ok(Self::Env(key, value))
        }
        AttributeDiscriminant::WorkingDirectory => Ok(Self::WorkingDirectory(
          arguments.into_iter().next().map(|(_, expr)| expr).unwrap(),
        )),
        _ => unreachable!(),
      };
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
      AttributeDiscriminant::Arg => {
        let arg = arguments.into_iter().next().unwrap();

        let (long, long_key) = keyword_arguments
          .remove("long")
          .map(|(key, expression)| {
            if let Some(expression) = expression {
              let literal = Self::require_string_literal(name, key, expression)?;
              Self::check_option_name(&arg, &literal)?;
              Ok((Some(literal), None))
            } else {
              Ok((Some(arg.clone()), Some(*key)))
            }
          })
          .transpose()?
          .unwrap_or((None, None));

        let short = Self::remove_required(&mut keyword_arguments, "short")?
          .map(|(key, expression)| {
            let literal = Self::require_string_literal(name, key, expression)?;

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
          .map(|(key, expression)| {
            Pattern::new(&Self::require_string_literal(name, key, expression)?)
          })
          .transpose()?;

        let value = Self::remove_required(&mut keyword_arguments, "value")?
          .map(|(key, expression)| {
            if long.is_none() && short.is_none() {
              return Err(
                key.error(CompileErrorKind::ArgAttributeRequiresOption { keyword: "value" }),
              );
            }
            Ok(expression)
          })
          .transpose()?;

        let flag = keyword_arguments
          .remove("flag")
          .map(|(key, expression)| {
            if expression.is_some() {
              return Err(key.error(CompileErrorKind::FlagAttributeTakesNoValue {
                parameter: arg.cooked.clone(),
              }));
            }
            if long.is_none() && short.is_none() {
              return Err(
                key.error(CompileErrorKind::ArgAttributeRequiresOption { keyword: "flag" }),
              );
            }
            if value.is_some() {
              return Err(key.error(CompileErrorKind::FlagAndValueArgAttribute {
                parameter: arg.cooked.clone(),
              }));
            }
            Ok(*key)
          })
          .transpose()?;

        let help = Self::remove_required(&mut keyword_arguments, "help")?
          .map(|(key, expression)| Self::require_string_literal(name, key, expression))
          .transpose()?;

        Self::Arg {
          flag,
          help,
          long,
          long_key,
          name: arg,
          pattern,
          short,
          value,
        }
      }
      AttributeDiscriminant::Android => Self::Android,
      AttributeDiscriminant::Cache => Self::Cache {
        inputs: Self::remove_required(&mut keyword_arguments, "inputs")?
          .map(|(_key, expression)| expression),
        outputs: Self::remove_required(&mut keyword_arguments, "outputs")?
          .map(|(_key, expression)| expression),
      },
      AttributeDiscriminant::Continue => Self::Continue(
        arguments
          .into_iter()
          .map(|literal| {
            Signal::from_name(&literal.cooked).ok_or_else(|| {
              literal.token.error(CompileErrorKind::InvalidSignal {
                signal: literal.cooked.clone(),
              })
            })
          })
          .collect::<CompileResult<BTreeSet<Signal>>>()?,
      ),
      AttributeDiscriminant::Confirm
      | AttributeDiscriminant::Env
      | AttributeDiscriminant::WorkingDirectory => unreachable!(),
      AttributeDiscriminant::Default => Self::Default,
      AttributeDiscriminant::Doc => Self::Doc(arguments.into_iter().next()),
      AttributeDiscriminant::Dragonfly => Self::Dragonfly,
      AttributeDiscriminant::ExitMessage => Self::ExitMessage,
      AttributeDiscriminant::Extension => Self::Extension(arguments.into_iter().next().unwrap()),
      AttributeDiscriminant::Freebsd => Self::Freebsd,
      AttributeDiscriminant::Group => Self::Group(arguments.into_iter().next().unwrap()),
      AttributeDiscriminant::Linux => Self::Linux,
      AttributeDiscriminant::Macos => Self::Macos,
      AttributeDiscriminant::Metadata => Self::Metadata(arguments),
      AttributeDiscriminant::Netbsd => Self::Netbsd,
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
      AttributeDiscriminant::Shell => Self::Shell,
      AttributeDiscriminant::Unix => Self::Unix,
      AttributeDiscriminant::Windows => Self::Windows,
    };

    if let Some((_name, (key, _literal))) = keyword_arguments.pop_first() {
      return Err(key.error(CompileErrorKind::UnknownAttributeKey {
        attribute: name.lexeme(),
        key: key.lexeme(),
      }));
    }

    Ok(attribute)
  }

  fn remove_required(
    keyword_arguments: &mut BTreeMap<&'src str, (Name<'src>, Option<Expression<'src>>)>,
    key: &'src str,
  ) -> CompileResult<'src, Option<(Name<'src>, Expression<'src>)>> {
    let Some((key, expression)) = keyword_arguments.remove(key) else {
      return Ok(None);
    };

    let expression =
      expression.ok_or_else(|| key.error(CompileErrorKind::AttributeKeyMissingValue { key }))?;

    Ok(Some((key, expression)))
  }

  fn require_string_literal(
    attribute: Name<'src>,
    key: Name<'src>,
    expression: Expression<'src>,
  ) -> CompileResult<'src, StringLiteral<'src>> {
    let Expression::StringLiteral { string_literal } = expression else {
      return Err(key.error(CompileErrorKind::AttributeArgumentExpression {
        attribute: attribute.lexeme(),
      }));
    };

    Ok(string_literal)
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
        flag,
        help,
        long,
        long_key,
        name,
        pattern,
        short,
        value,
      } => {
        write!(f, "({name}")?;

        if long_key.is_some() {
          write!(f, ", long")?;
        } else if let Some(long) = long {
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

        if flag.is_some() {
          write!(f, ", flag")?;
        }

        if let Some(help) = help {
          write!(f, ", help={help}")?;
        }

        write!(f, ")")?;
      }
      Self::Android
      | Self::Confirm(None)
      | Self::Default
      | Self::Doc(None)
      | Self::Dragonfly
      | Self::ExitMessage
      | Self::Freebsd
      | Self::Linux
      | Self::Macos
      | Self::Netbsd
      | Self::NoCd
      | Self::NoExitMessage
      | Self::NoQuiet
      | Self::Openbsd
      | Self::Parallel
      | Self::PositionalArguments
      | Self::Private
      | Self::Script(None)
      | Self::Shell
      | Self::Unix
      | Self::Windows => {}
      Self::Cache { inputs, outputs } => {
        let mut arguments = Vec::new();
        if let Some(inputs) = inputs {
          arguments.push(format!("inputs={inputs}"));
        }
        if let Some(outputs) = outputs {
          arguments.push(format!("outputs={outputs}"));
        }
        if !arguments.is_empty() {
          write!(f, "({})", arguments.join(", "))?;
        }
      }
      Self::Confirm(Some(argument)) | Self::WorkingDirectory(argument) => {
        write!(f, "({argument})")?;
      }
      Self::Doc(Some(argument)) | Self::Extension(argument) | Self::Group(argument) => {
        write!(f, "({argument})")?;
      }
      Self::Continue(signals) => {
        if !signals.is_empty() {
          write!(f, "(")?;
          for (i, signal) in signals.iter().enumerate() {
            if i > 0 {
              write!(f, ", ")?;
            }
            write!(f, "\"{signal}\"")?;
          }
          write!(f, ")")?;
        }
      }
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
