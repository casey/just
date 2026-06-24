use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct CompileError<'src> {
  pub(crate) kind: Box<CompileErrorKind<'src>>,
  pub(crate) token: Token<'src>,
}

impl<'src> CompileError<'src> {
  pub(crate) fn context(&self) -> Token<'src> {
    self.token
  }

  pub(crate) fn new(token: Token<'src>, kind: CompileErrorKind<'src>) -> Self {
    Self {
      token,
      kind: kind.into(),
    }
  }

  pub(crate) fn source(&self) -> Option<&dyn std::error::Error> {
    match &*self.kind {
      CompileErrorKind::ArgumentPatternRegex { source } => Some(source),
      _ => None,
    }
  }
}

impl Display for CompileError<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use CompileErrorKind::*;

    match &*self.kind {
      ArgAttributeRequiresOption { keyword } => {
        write!(
          f,
          "argument attribute `{keyword}` only valid with `long` or `short`"
        )
      }
      ArgumentPatternRegex { .. } => {
        write!(f, "failed to parse argument pattern")
      }
      AttributeArgumentCountMismatch {
        attribute,
        found,
        min,
        max,
      } => {
        write!(
          f,
          "attribute `{attribute}` got {} but takes ",
          Count::numbered("argument", found),
        )?;

        if min == max {
          write!(f, "{}", Count::numbered("argument", min))
        } else if found < min {
          write!(f, "at least {}", Count::numbered("argument", min))
        } else {
          write!(f, "at most {}", Count::numbered("argument", max))
        }
      }
      AttributeArgumentExpression { attribute } => {
        write!(
          f,
          "attribute `{attribute}` arguments must be string literals"
        )
      }
      AttributePositionalFollowsKeyword => {
        write!(
          f,
          "positional attribute arguments cannot follow keyword attribute arguments"
        )
      }
      BacktickShebang => write!(f, "backticks may not start with `#!`"),
      CircularRecipeDependency { recipe, circle } => {
        if circle.len() == 2 {
          write!(f, "recipe `{recipe}` depends on itself")
        } else {
          write!(
            f,
            "recipe `{recipe}` has circular dependency `{}`",
            circle.join(" -> ")
          )
        }
      }
      CircularVariableDependency { variable, circle } => {
        if circle.len() == 2 {
          write!(f, "variable `{variable}` is defined in terms of itself")
        } else {
          write!(
            f,
            "variable `{variable}` depends on its own value: `{}`",
            circle.join(" -> "),
          )
        }
      }
      DependencyArgumentCountMismatch {
        dependency,
        found,
        min,
        max,
      } => {
        write!(
          f,
          "dependency `{dependency}` got {} but takes ",
          Count::numbered("argument", found),
        )?;

        if min == max {
          write!(f, "{}", Count::numbered("argument", min))
        } else if found < min {
          write!(f, "at least {}", Count::numbered("argument", min))
        } else {
          write!(f, "at most {}", Count::numbered("argument", max))
        }
      }
      DuplicateArgAttribute { arg, first } => write!(
        f,
        "recipe attribute for argument `{arg}` first used on line {} is duplicated on line {}",
        first.ordinal(),
        self.token.line.ordinal(),
      ),
      DuplicateAttribute { attribute, first } => write!(
        f,
        "recipe attribute `{attribute}` first used on line {} is duplicated on line {}",
        first.ordinal(),
        self.token.line.ordinal(),
      ),
      DuplicateAttributeKey { attribute, key } => {
        write!(f, "duplicate key `{key}` for `{attribute}` attribute")
      }
      DuplicateDefault { recipe } => write!(
        f,
        "recipe `{recipe}` has duplicate `[default]` attribute, which may only appear once per module",
      ),
      DuplicateOption { recipe, option } => {
        write!(
          f,
          "recipe `{recipe}` defines option `{option}` multiple times"
        )
      }
      DuplicateParameter { recipe, parameter } => {
        write!(f, "recipe `{recipe}` has duplicate parameter `{parameter}`")
      }
      DuplicateSet { setting, first } => write!(
        f,
        "setting `{setting}` first set on line {} is redefined on line {}",
        first.ordinal(),
        self.token.line.ordinal(),
      ),
      DuplicateVariable { variable } => {
        write!(f, "variable `{variable}` has multiple definitions")
      }
      DuplicateUnexport { variable } => {
        write!(f, "variable `{variable}` is unexported multiple times")
      }
      ExitMessageAndNoExitMessageAttribute { recipe } => write!(
        f,
        "recipe `{recipe}` has both `[exit-message]` and `[no-exit-message]` attributes"
      ),
      ExpectedKeyword { expected, found } => {
        let expected = List::or_ticked(expected);
        if found.kind == TokenKind::Identifier {
          write!(
            f,
            "expected keyword {expected} but found identifier `{}`",
            found.lexeme()
          )
        } else {
          write!(f, "expected keyword {expected} but found `{}`", found.kind)
        }
      }
      ExportUnexported { variable } => {
        write!(f, "variable {variable} is both exported and unexported")
      }
      ExtraLeadingWhitespace => write!(f, "recipe line has extra leading whitespace"),
      ExtraneousAttributes { count } => {
        write!(f, "extraneous {}", Count::unnumbered("attribute", count))
      }
      FlagAndValueArgAttribute { parameter } => {
        write!(
          f,
          "argument `{parameter}` may not have both `flag` and `value` attributes"
        )
      }
      FlagAttributeTakesNoValue { parameter } => {
        write!(
          f,
          "`flag` attribute for argument `{parameter}` takes no value"
        )
      }
      FlagWithDefault { parameter } => {
        write!(f, "flag parameter `{parameter}` may not have a default")
      }
      FunctionArgumentCountMismatch {
        function,
        arguments,
        expected,
      } => write!(
        f,
        "function `{function}` called with {} but takes {}",
        Count::numbered("argument", arguments),
        expected.display(),
      ),
      GuardAndInfallibleSigil => write!(
        f,
        "the guard `?` and infallible `-` sigils may not be used together"
      ),
      Include => write!(
        f,
        "the `!include` directive has been stabilized as `import`"
      ),
      IncompatibleSettings {
        first,
        first_line,
        second,
      } => write!(
        f,
        "`{}` set on line {} is incompatible with `{}`",
        first.lexeme(),
        first_line.ordinal(),
        second.lexeme()
      ),
      InconsistentLeadingWhitespace { expected, found } => write!(
        f,
        "recipe line has inconsistent leading whitespace, started with `{}` but found line with \
          `{}`",
        ShowWhitespace(expected),
        ShowWhitespace(found)
      ),
      Internal { message } => write!(
        f,
        "internal error, this may indicate a bug in just: {message}\n\
           consider filing an issue: https://github.com/casey/just/issues/new"
      ),
      InvalidAttribute {
        item_name,
        item_kind,
        attribute,
      } => write!(
        f,
        "{item_kind} `{item_name}` has invalid attribute `{}`",
        attribute.name(),
      ),
      InvalidEscapeSequence { character } => write!(
        f,
        "`\\{}` is not a valid escape sequence",
        match character {
          '`' => "\\`".to_owned(),
          '\\' => "\\".to_owned(),
          '\'' => "'".to_owned(),
          '"' => "\"".to_owned(),
          _ => character.escape_default().collect(),
        }
      ),
      InvalidShellRecipeAttribute { attribute, recipe } => write!(
        f,
        "shell recipe `{recipe}` has script recipe attribute `{}`",
        attribute.name(),
      ),
      InvalidMinimumVersion { source, version } => {
        write!(
          f,
          "setting `minimum-version` has invalid version `{version}`: {source}"
        )
      }
      InvalidSignal { signal } => write!(
        f,
        "invalid signal `{signal}`: expected `SIGHUP`, `SIGINT`, or `SIGQUIT`"
      ),
      ListFeature(feature) => write!(f, "{feature}"),
      MappedDependencyMultipleStarredArguments => {
        write!(
          f,
          "mapped dependencies may not have multiple starred arguments"
        )
      }
      MappedDependencyWithoutListsSetting => {
        write!(f, "mapped dependencies require `set lists`")
      }
      MappedDependencyWithoutStarredArgument => {
        write!(f, "mapped dependencies must have starred argument")
      }
      MinimumVersion { current, minimum } => write!(
        f,
        "justfile requires just {minimum} or later, but using {current}",
      ),
      MinimumVersionExpression => write!(
        f,
        "setting `minimum-version` value must be a plain string literal"
      ),
      MismatchedClosingDelimiter {
        open,
        open_line,
        close,
      } => write!(
        f,
        "mismatched closing delimiter `{}`, did you mean to close the `{}` on line {}?",
        close.close(),
        open.open(),
        open_line.ordinal(),
      ),
      MixedLeadingWhitespace { whitespace } => write!(
        f,
        "found a mix of tabs and spaces in leading whitespace: `{}`\nleading whitespace may \
           consist of tabs or spaces, but not both",
        ShowWhitespace(whitespace)
      ),
      NoCdAndWorkingDirectoryAttribute { recipe } => write!(
        f,
        "recipe `{recipe}` has both `[no-cd]` and `[working-directory]` attributes"
      ),
      OptionNameContainsEqualSign { parameter } => {
        write!(
          f,
          "option name for parameter `{parameter}` contains equal sign"
        )
      }
      OptionNameEmpty { parameter } => {
        write!(f, "option name for parameter `{parameter}` is empty")
      }
      ParameterFollowsVariadicParameter { parameter } => {
        write!(f, "parameter `{parameter}` follows variadic parameter")
      }
      ParsingRecursionDepthExceeded => write!(f, "parsing recursion depth exceeded"),
      Redefinition {
        first,
        first_type,
        name,
        second_type,
      } => {
        if first_type == second_type {
          write!(
            f,
            "{first_type} `{name}` first defined on line {} is redefined on line {}",
            first.ordinal(),
            self.token.line.ordinal(),
          )
        } else {
          write!(
            f,
            "{first_type} `{name}` defined on line {} is redefined as {} {second_type} on line {}",
            first.ordinal(),
            if *second_type == "alias" { "an" } else { "a" },
            self.token.line.ordinal(),
          )
        }
      }
      ScriptAndShellAttribute { recipe } => write!(
        f,
        "recipe `{recipe}` has both `[script]` and `[shell]` attributes"
      ),
      ShellExpansion { err } => write!(f, "shell expansion failed: {err}"),
      ShortOptionWithMultipleCharacters { parameter } => {
        write!(
          f,
          "short option name for parameter `{parameter}` contains multiple characters"
        )
      }
      StarredArgumentOutsideMappedDependency => write!(
        f,
        "starred arguments may not be used outside mapped dependencies",
      ),
      RequiredParameterFollowsDefaultParameter { parameter } => write!(
        f,
        "non-default parameter `{parameter}` follows default parameter"
      ),
      UndefinedArgAttribute { argument } => {
        write!(f, "argument attribute for undefined argument `{argument}`")
      }
      UndefinedFunction { function } => write!(f, "call to undefined function `{function}`"),
      UndefinedVariable { variable } => write!(f, "variable `{variable}` not defined"),
      UnexpectedCharacter { expected } => {
        write!(f, "expected character {}", List::or_ticked(expected))
      }
      UnexpectedClosingDelimiter { close } => {
        write!(f, "unexpected closing delimiter `{}`", close.close())
      }
      UnexpectedEndOfToken { expected } => {
        write!(
          f,
          "expected character {} but found end-of-file",
          List::or_ticked(expected),
        )
      }
      UnexpectedToken { expected, found } => {
        write!(f, "expected {}, but found {found}", List::or(expected))
      }
      UnicodeEscapeCharacter { character } => {
        write!(f, "expected hex digit [0-9A-Fa-f] but found `{character}`")
      }
      UnicodeEscapeDelimiter { character } => write!(
        f,
        "expected unicode escape sequence delimiter `{{` but found `{character}`"
      ),
      UnicodeEscapeEmpty => write!(f, "unicode escape sequences must not be empty"),
      UnicodeEscapeLength { hex } => write!(
        f,
        "unicode escape sequence starting with `\\u{{{hex}` longer than six hex digits"
      ),
      UnicodeEscapeRange { hex } => {
        write!(
          f,
          "unicode escape sequence value `{hex}` greater than maximum valid code point `10FFFF`",
        )
      }
      UnicodeEscapeUnterminated => write!(f, "unterminated unicode escape sequence"),
      UnknownAliasTarget { alias, target } => {
        write!(f, "alias `{alias}` has an unknown target `{target}`")
      }
      AttributeKeyMissingValue { key } => {
        write!(f, "attribute key `{key}` requires value")
      }
      UnknownAttributeKey { attribute, key } => {
        write!(f, "unknown key `{key}` for `{attribute}` attribute")
      }
      UnknownAttribute { attribute } => write!(f, "unknown attribute `{attribute}`"),
      UnknownDependency { recipe, unknown } => {
        write!(f, "recipe `{recipe}` has unknown dependency `{unknown}`")
      }
      UnknownSetting { setting } => write!(f, "unknown setting `{setting}`"),
      UnknownStartOfToken { start } => {
        write!(f, "unknown start of token '{start}'")?;
        if !start.is_ascii_graphic() {
          write!(f, " (U+{:04X})", *start as u32)?;
        }
        Ok(())
      }
      UnpairedCarriageReturn => write!(f, "unpaired carriage return"),
      UnterminatedBacktick => write!(f, "unterminated backtick"),
      UnterminatedInterpolation => write!(f, "unterminated interpolation"),
      UnterminatedString => write!(f, "unterminated string"),
      VariadicParameterWithOption => write!(f, "variadic parameters may not be options"),
    }
  }
}
