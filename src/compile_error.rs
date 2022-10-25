use super::*;

#[derive(Debug, PartialEq)]
pub(crate) struct CompileError<'src> {
  pub(crate) token: Token<'src>,
  pub(crate) kind: Box<CompileErrorKind<'src>>,
}

impl<'src> CompileError<'src> {
  pub(crate) fn context(&self) -> Token<'src> {
    self.token
  }

  pub(crate) fn new(token: Token<'src>, kind: CompileErrorKind<'src>) -> CompileError<'src> {
    Self {
      token,
      kind: Box::new(kind),
    }
  }
}

impl Display for CompileError<'_> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    use CompileErrorKind::*;

    match &*self.kind {
      AliasShadowsRecipe { alias, recipe_line } => {
        write!(
          f,
          "Alias `{}` defined on line {} shadows recipe `{}` defined on line {}",
          alias,
          self.token.line.ordinal(),
          alias,
          recipe_line.ordinal(),
        )?;
      }
      BacktickShebang => {
        write!(f, "Backticks may not start with `#!`")?;
      }
      CircularRecipeDependency { recipe, ref circle } => {
        if circle.len() == 2 {
          write!(f, "Recipe `{}` depends on itself", recipe)?;
        } else {
          write!(
            f,
            "Recipe `{}` has circular dependency `{}`",
            recipe,
            circle.join(" -> ")
          )?;
        }
      }
      CircularVariableDependency {
        variable,
        ref circle,
      } => {
        if circle.len() == 2 {
          write!(f, "Variable `{}` is defined in terms of itself", variable)?;
        } else {
          write!(
            f,
            "Variable `{}` depends on its own value: `{}`",
            variable,
            circle.join(" -> ")
          )?;
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
          "Dependency `{}` got {} {} but takes ",
          dependency,
          found,
          Count("argument", *found),
        )?;

        if min == max {
          let expected = min;
          write!(f, "{} {}", expected, Count("argument", *expected))?;
        } else if found < min {
          write!(f, "at least {} {}", min, Count("argument", *min))?;
        } else {
          write!(f, "at most {} {}", max, Count("argument", *max))?;
        }
      }
      DuplicateAlias { alias, first } => {
        write!(
          f,
          "Alias `{}` first defined on line {} is redefined on line {}",
          alias,
          first.ordinal(),
          self.token.line.ordinal(),
        )?;
      }
      DuplicateParameter { recipe, parameter } => {
        write!(
          f,
          "Recipe `{}` has duplicate parameter `{}`",
          recipe, parameter
        )?;
      }
      DuplicateRecipe { recipe, first } => {
        write!(
          f,
          "Recipe `{}` first defined on line {} is redefined on line {}",
          recipe,
          first.ordinal(),
          self.token.line.ordinal()
        )?;
      }
      DuplicateSet { setting, first } => {
        write!(
          f,
          "Setting `{}` first set on line {} is redefined on line {}",
          setting,
          first.ordinal(),
          self.token.line.ordinal(),
        )?;
      }
      DuplicateVariable { variable } => {
        write!(f, "Variable `{}` has multiple definitions", variable)?;
      }
      ExpectedKeyword { expected, found } => {
        if found.kind == TokenKind::Identifier {
          write!(
            f,
            "Expected keyword {} but found identifier `{}`",
            List::or_ticked(expected),
            found.lexeme()
          )?;
        } else {
          write!(
            f,
            "Expected keyword {} but found `{}`",
            List::or_ticked(expected),
            found.kind
          )?;
        }
      }
      ExtraLeadingWhitespace => {
        write!(f, "Recipe line has extra leading whitespace")?;
      }
      FunctionArgumentCountMismatch {
        function,
        found,
        expected,
      } => {
        write!(
          f,
          "Function `{}` called with {} {} but takes {}",
          function,
          found,
          Count("argument", *found),
          expected.display(),
        )?;
      }
      InconsistentLeadingWhitespace { expected, found } => {
        write!(
          f,
          "Recipe line has inconsistent leading whitespace. Recipe started with `{}` but found \
           line with `{}`",
          ShowWhitespace(expected),
          ShowWhitespace(found)
        )?;
      }
      Internal { ref message } => {
        write!(
          f,
          "Internal error, this may indicate a bug in just: {}\n\
           consider filing an issue: https://github.com/casey/just/issues/new",
          message
        )?;
      }
      InvalidEscapeSequence { character } => {
        let representation = match character {
          '`' => r"\`".to_owned(),
          '\\' => r"\".to_owned(),
          '\'' => r"'".to_owned(),
          '"' => r#"""#.to_owned(),
          _ => character.escape_default().collect(),
        };
        write!(f, "`\\{}` is not a valid escape sequence", representation)?;
      }
      MismatchedClosingDelimiter {
        open,
        open_line,
        close,
      } => {
        write!(
          f,
          "Mismatched closing delimiter `{}`. (Did you mean to close the `{}` on line {}?)",
          close.close(),
          open.open(),
          open_line.ordinal(),
        )?;
      }
      MixedLeadingWhitespace { whitespace } => {
        write!(
          f,
          "Found a mix of tabs and spaces in leading whitespace: `{}`\nLeading whitespace may \
           consist of tabs or spaces, but not both",
          ShowWhitespace(whitespace)
        )?;
      }
      ParameterFollowsVariadicParameter { parameter } => {
        write!(f, "Parameter `{}` follows variadic parameter", parameter)?;
      }
      ParameterShadowsVariable { parameter } => {
        write!(
          f,
          "Parameter `{}` shadows variable of the same name",
          parameter
        )?;
      }
      ParsingRecursionDepthExceeded => {
        write!(f, "Parsing recursion depth exceeded")?;
      }
      RequiredParameterFollowsDefaultParameter { parameter } => {
        write!(
          f,
          "Non-default parameter `{}` follows default parameter",
          parameter
        )?;
      }
      UndefinedVariable { variable } => {
        write!(f, "Variable `{}` not defined", variable)?;
      }
      UnexpectedCharacter { expected } => {
        write!(f, "Expected character `{}`", expected)?;
      }
      UnexpectedClosingDelimiter { close } => {
        write!(f, "Unexpected closing delimiter `{}`", close.close())?;
      }
      UnexpectedEndOfToken { expected } => {
        write!(f, "Expected character `{}` but found end-of-file", expected)?;
      }
      UnexpectedToken {
        ref expected,
        found,
      } => {
        write!(f, "Expected {}, but found {}", List::or(expected), found)?;
      }
      UnknownAliasTarget { alias, target } => {
        write!(f, "Alias `{}` has an unknown target `{}`", alias, target)?;
      }
      UnknownAttribute { attribute } => {
        write!(f, "Unknown attribute `{}`", attribute)?;
      }
      UnknownDependency { recipe, unknown } => {
        write!(
          f,
          "Recipe `{}` has unknown dependency `{}`",
          recipe, unknown
        )?;
      }
      UnknownFunction { function } => {
        write!(f, "Call to unknown function `{}`", function)?;
      }
      UnknownSetting { setting } => {
        write!(f, "Unknown setting `{}`", setting)?;
      }
      UnknownStartOfToken => {
        write!(f, "Unknown start of token:")?;
      }
      UnpairedCarriageReturn => {
        write!(f, "Unpaired carriage return")?;
      }
      UnterminatedBacktick => {
        write!(f, "Unterminated backtick")?;
      }
      UnterminatedInterpolation => {
        write!(f, "Unterminated interpolation")?;
      }
      UnterminatedString => {
        write!(f, "Unterminated string")?;
      }
    }

    Ok(())
  }
}
