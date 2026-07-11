use super::*;

#[allow(clippy::large_enum_variant)]
#[derive(
  Clone, Debug, EnumDiscriminants, Eq, IntoStaticStr, Ord, PartialEq, PartialOrd, Serialize,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[strum_discriminants(name(AttributeKind))]
#[strum_discriminants(derive(EnumString, Ord, PartialOrd))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub(crate) enum Attribute<'src> {
  Android,
  Arg {
    #[serde(skip)]
    flag: Option<Token<'src>>,
    help: Option<String>,
    #[serde(skip)]
    help_property: Option<(Name<'src>, Expression<'src>)>,
    long: Option<StringLiteral<'src>>,
    #[serde(skip)]
    long_key: Option<Name<'src>>,
    max: Option<u64>,
    #[serde(skip)]
    max_key: Option<Name<'src>>,
    min: Option<u64>,
    #[serde(skip)]
    min_key: Option<Name<'src>>,
    #[serde(skip)]
    multiple: Option<Token<'src>>,
    name: StringLiteral<'src>,
    pattern: Option<Pattern>,
    #[serde(skip)]
    pattern_property: Option<(Name<'src>, Expression<'src>)>,
    short: Option<StringLiteral<'src>>,
    #[serde(skip)]
    short_key: Option<Name<'src>>,
    value: Option<Expression<'src>>,
  },
  Cache {
    extra: Option<Expression<'src>>,
    inputs: Option<Expression<'src>>,
    outputs: Option<Expression<'src>>,
  },
  Confirm(Option<Expression<'src>>),
  Continue(BTreeSet<Signal>),
  Default,
  Doc(Option<Expression<'src>>),
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

impl AttributeKind {
  fn accepts_expressions(self) -> bool {
    matches!(
      self,
      Self::Confirm | Self::Doc | Self::Env | Self::WorkingDirectory
    )
  }

  pub(crate) fn accepts_keyword_arguments(self) -> bool {
    matches!(self, Self::Arg | Self::Cache)
  }

  pub(crate) fn is_enabler(self) -> bool {
    matches!(
      self,
      Self::Android
        | Self::Dragonfly
        | Self::Freebsd
        | Self::Linux
        | Self::Macos
        | Self::Netbsd
        | Self::Openbsd
        | Self::Unix
        | Self::Windows
    )
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

    if literal.cooked.starts_with('-') {
      return Err(
        literal
          .token
          .error(CompileErrorKind::OptionNameStartsWithDash {
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
    kind: AttributeKind,
    arguments: Vec<(Token<'src>, Expression<'src>)>,
    mut keyword_arguments: BTreeMap<&'src str, (Name<'src>, Option<Expression<'src>>)>,
  ) -> CompileResult<'src, Self> {
    let found = arguments.len();
    let range = kind.argument_range();
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

    if kind.accepts_expressions() {
      if let Some((_name, (key, _literal))) = keyword_arguments.pop_first() {
        return Err(key.error(CompileErrorKind::UnknownAttributeKey {
          attribute: name.lexeme(),
          key: key.lexeme(),
        }));
      }

      return match kind {
        AttributeKind::Confirm => Ok(Self::Confirm(
          arguments.into_iter().next().map(|(_, expr)| expr),
        )),
        AttributeKind::Doc => Ok(Self::Doc(
          arguments.into_iter().next().map(|(_, expr)| expr),
        )),
        AttributeKind::Env => {
          let mut arguments = arguments.into_iter();
          let (_, key) = arguments.next().unwrap();
          let (_, value) = arguments.next().unwrap();
          Ok(Self::Env(key, value))
        }
        AttributeKind::WorkingDirectory => Ok(Self::WorkingDirectory(
          arguments.into_iter().next().map(|(_, expr)| expr).unwrap(),
        )),
        _ => unreachable!(),
      };
    }

    let arguments = arguments
      .into_iter()
      .map(|(token, argument)| {
        let Expression::StringLiteral { string_literal } = argument else {
          return Err(
            token.error(CompileErrorKind::AttributeArgumentExpression { attribute: name }),
          );
        };
        Ok(string_literal)
      })
      .collect::<CompileResult<Vec<StringLiteral>>>()?;

    let attribute = match kind {
      AttributeKind::Arg => Self::new_arg(name, arguments, &mut keyword_arguments)?,
      AttributeKind::Android => Self::Android,
      AttributeKind::Cache => Self::Cache {
        extra: Self::remove_required(&mut keyword_arguments, "extra")?
          .map(|(_key, expression)| expression),
        inputs: Self::remove_required(&mut keyword_arguments, "inputs")?
          .map(|(_key, expression)| expression),
        outputs: Self::remove_required(&mut keyword_arguments, "outputs")?
          .map(|(_key, expression)| expression),
      },
      AttributeKind::Continue => Self::Continue(
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
      AttributeKind::Confirm
      | AttributeKind::Doc
      | AttributeKind::Env
      | AttributeKind::WorkingDirectory => {
        unreachable!()
      }
      AttributeKind::Default => Self::Default,
      AttributeKind::Dragonfly => Self::Dragonfly,
      AttributeKind::ExitMessage => Self::ExitMessage,
      AttributeKind::Extension => Self::Extension(arguments.into_iter().next().unwrap()),
      AttributeKind::Freebsd => Self::Freebsd,
      AttributeKind::Group => Self::Group(arguments.into_iter().next().unwrap()),
      AttributeKind::Linux => Self::Linux,
      AttributeKind::Macos => Self::Macos,
      AttributeKind::Metadata => Self::Metadata(arguments),
      AttributeKind::Netbsd => Self::Netbsd,
      AttributeKind::NoCd => Self::NoCd,
      AttributeKind::NoExitMessage => Self::NoExitMessage,
      AttributeKind::NoQuiet => Self::NoQuiet,
      AttributeKind::Openbsd => Self::Openbsd,
      AttributeKind::Parallel => Self::Parallel,
      AttributeKind::PositionalArguments => Self::PositionalArguments,
      AttributeKind::Private => Self::Private,
      AttributeKind::Script => Self::Script({
        let mut arguments = arguments.into_iter();
        arguments.next().map(|command| Interpreter {
          command,
          arguments: arguments.collect(),
        })
      }),
      AttributeKind::Shell => Self::Shell,
      AttributeKind::Unix => Self::Unix,
      AttributeKind::Windows => Self::Windows,
    };

    if let Some((_name, (key, _literal))) = keyword_arguments.pop_first() {
      return Err(key.error(CompileErrorKind::UnknownAttributeKey {
        attribute: name.lexeme(),
        key: key.lexeme(),
      }));
    }

    Ok(attribute)
  }

  fn new_arg(
    name: Name<'src>,
    arguments: Vec<StringLiteral<'src>>,
    keyword_arguments: &mut BTreeMap<&'src str, (Name<'src>, Option<Expression<'src>>)>,
  ) -> CompileResult<'src, Self> {
    static NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new("^(0|[1-9][0-9]*)$").unwrap());

    let arg = arguments.into_iter().next().unwrap();

    let (long, long_key) = keyword_arguments
      .remove("long")
      .map(|(key, expression)| {
        if let Some(expression) = expression {
          let literal = Self::require_string_literal(name, key, expression)?;
          Self::check_option_name(&arg, &literal)?;
          Ok((Some(literal), None))
        } else {
          Ok((Some(arg.clone()), Some(key)))
        }
      })
      .transpose()?
      .unwrap_or_default();

    let (short, short_key) =
      keyword_arguments
        .remove("short")
        .map(|(key, expression)| {
          if let Some(expression) = expression {
            let literal = Self::require_string_literal(name, key, expression)?;

            Self::check_option_name(&arg, &literal)?;

            if literal.cooked.chars().count() != 1 {
              return Err(literal.token.error(
                CompileErrorKind::ShortOptionWithMultipleCharacters {
                  parameter: arg.cooked.clone(),
                },
              ));
            }

            Ok((Some(literal), None))
          } else {
            Ok((Some(arg.clone()), Some(key)))
          }
        })
        .transpose()?
        .unwrap_or_default();

    let pattern_property = Self::remove_required(keyword_arguments, "pattern")?;

    let value = Self::remove_required(keyword_arguments, "value")?
      .map(|(key, expression)| {
        if long.is_none() && short.is_none() {
          return Err(key.error(CompileErrorKind::ArgAttributeRequiresOption { key }));
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
          return Err(key.error(CompileErrorKind::ArgAttributeRequiresOption { key }));
        }
        if value.is_some() {
          return Err(key.error(CompileErrorKind::FlagAndValueArgAttribute {
            parameter: arg.cooked.clone(),
          }));
        }
        if pattern_property.is_some() {
          return Err(key.error(CompileErrorKind::FlagAndPatternArgAttribute {
            parameter: arg.cooked.clone(),
          }));
        }
        Ok(*key)
      })
      .transpose()?;

    let multiple = keyword_arguments
      .remove("multiple")
      .map(|(key, expression)| {
        if expression.is_some() {
          return Err(key.error(CompileErrorKind::AttributeKeyTakesNoValue { key }));
        }
        if long.is_none() && short.is_none() {
          return Err(key.error(CompileErrorKind::ArgAttributeRequiresOption { key }));
        }
        Ok(*key)
      })
      .transpose()?;

    let (max, max_key) = Self::remove_required(keyword_arguments, "max")?
      .map(|(key, expression)| {
        let literal = Self::require_string_literal(name, key, expression)?;

        if !NUMBER.is_match(&literal.cooked) {
          return Err(literal.token.error(CompileErrorKind::ArgumentCountValue {
            key,
            value: literal.cooked.clone(),
          }));
        }

        let max = literal.cooked.parse::<u64>().map_err(|source| {
          literal.token.error(CompileErrorKind::ArgumentCountParse {
            key,
            value: literal.cooked.clone(),
            source,
          })
        })?;

        Ok((Some(max), Some(key)))
      })
      .transpose()?
      .unwrap_or_default();

    let (min, min_key) = Self::remove_required(keyword_arguments, "min")?
      .map(|(key, expression)| {
        let literal = Self::require_string_literal(name, key, expression)?;

        if !NUMBER.is_match(&literal.cooked) {
          return Err(literal.token.error(CompileErrorKind::ArgumentCountValue {
            key,
            value: literal.cooked.clone(),
          }));
        }

        let min = literal.cooked.parse::<u64>().map_err(|source| {
          literal.token.error(CompileErrorKind::ArgumentCountParse {
            key,
            value: literal.cooked.clone(),
            source,
          })
        })?;

        Ok((Some(min), Some(key)))
      })
      .transpose()?
      .unwrap_or_default();

    if let (Some(min), Some(max)) = (min, max)
      && min > max
    {
      return Err(
        min_key
          .unwrap()
          .error(CompileErrorKind::ArgAttributeMinExceedsMax { min, max }),
      );
    }

    let help_property = Self::remove_required(keyword_arguments, "help")?;

    Ok(Self::Arg {
      flag,
      help: None,
      help_property,
      long,
      long_key,
      max,
      max_key,
      min,
      min_key,
      multiple,
      name: arg,
      pattern: None,
      pattern_property,
      short,
      short_key,
      value,
    })
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
      return Err(key.error(CompileErrorKind::AttributeArgumentExpression { attribute }));
    };

    Ok(string_literal)
  }

  pub(crate) fn kind(&self) -> AttributeKind {
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
        help: _,
        help_property,
        long,
        long_key,
        max,
        max_key: _,
        min,
        min_key: _,
        multiple,
        name,
        pattern: _,
        pattern_property,
        short,
        short_key,
        value,
      } => {
        write!(f, "({name}")?;

        if long_key.is_some() {
          write!(f, ", long")?;
        } else if let Some(long) = long {
          write!(f, ", long={long}")?;
        }

        if short_key.is_some() {
          write!(f, ", short")?;
        } else if let Some(short) = short {
          write!(f, ", short={short}")?;
        }

        if let Some((_key, pattern)) = pattern_property {
          write!(f, ", pattern={pattern}")?;
        }

        if let Some(value) = value {
          write!(f, ", value={value}")?;
        }

        if flag.is_some() {
          write!(f, ", flag")?;
        }

        if multiple.is_some() {
          write!(f, ", multiple")?;
        }

        if let Some(min) = min {
          write!(f, ", min='{min}'")?;
        }

        if let Some(max) = max {
          write!(f, ", max='{max}'")?;
        }

        if let Some((_key, help)) = help_property {
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
      Self::Cache {
        extra,
        inputs,
        outputs,
      } => {
        let mut arguments = Vec::new();
        if let Some(extra) = extra {
          arguments.push(format!("extra={extra}"));
        }
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
      Self::Confirm(Some(argument))
      | Self::Doc(Some(argument))
      | Self::WorkingDirectory(argument) => {
        write!(f, "({argument})")?;
      }
      Self::Extension(argument) | Self::Group(argument) => {
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
