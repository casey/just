use crate::common::*;

#[derive(Debug, PartialEq)]
pub(crate) struct CompilationError<'a> {
  pub(crate) src: &'a str,
  pub(crate) offset: usize,
  pub(crate) line: usize,
  pub(crate) column: usize,
  pub(crate) width: usize,
  pub(crate) kind: CompilationErrorKind<'a>,
}

impl Error for CompilationError<'_> {}

impl Display for CompilationError<'_> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    use CompilationErrorKind::*;
    let message = Color::fmt(f).message();

    write!(f, "{}", message.prefix())?;

    match self.kind {
      AliasShadowsRecipe { alias, recipe_line } => {
        writeln!(
          f,
          "Alias `{}` defined on line {} shadows recipe `{}` defined on line {}",
          alias,
          self.line.ordinal(),
          alias,
          recipe_line.ordinal(),
        )?;
      }
      CircularRecipeDependency { recipe, ref circle } => {
        if circle.len() == 2 {
          writeln!(f, "Recipe `{}` depends on itself", recipe)?;
        } else {
          writeln!(
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
          writeln!(f, "Variable `{}` is defined in terms of itself", variable)?;
        } else {
          writeln!(
            f,
            "Variable `{}` depends on its own value: `{}`",
            variable,
            circle.join(" -> ")
          )?;
        }
      }

      InvalidEscapeSequence { character } => {
        let representation = match character {
          '`' => r"\`".to_string(),
          '\\' => r"\".to_string(),
          '\'' => r"'".to_string(),
          '"' => r#"""#.to_string(),
          _ => character.escape_default().collect(),
        };
        writeln!(f, "`\\{}` is not a valid escape sequence", representation)?;
      }
      DuplicateParameter { recipe, parameter } => {
        writeln!(
          f,
          "Recipe `{}` has duplicate parameter `{}`",
          recipe, parameter
        )?;
      }
      DuplicateVariable { variable } => {
        writeln!(f, "Variable `{}` has multiple definitions", variable)?;
      }
      UnexpectedToken {
        ref expected,
        found,
      } => {
        writeln!(f, "Expected {}, but found {}", List::or(expected), found)?;
      }
      DuplicateAlias { alias, first } => {
        writeln!(
          f,
          "Alias `{}` first defined on line {} is redefined on line {}",
          alias,
          first.ordinal(),
          self.line.ordinal(),
        )?;
      }
      DuplicateDependency { recipe, dependency } => {
        writeln!(
          f,
          "Recipe `{}` has duplicate dependency `{}`",
          recipe, dependency
        )?;
      }
      DuplicateRecipe { recipe, first } => {
        writeln!(
          f,
          "Recipe `{}` first defined on line {} is redefined on line {}",
          recipe,
          first.ordinal(),
          self.line.ordinal()
        )?;
      }
      DependencyHasParameters { recipe, dependency } => {
        writeln!(
          f,
          "Recipe `{}` depends on `{}` which requires arguments. \
           Dependencies may not require arguments",
          recipe, dependency
        )?;
      }
      ParameterShadowsVariable { parameter } => {
        writeln!(
          f,
          "Parameter `{}` shadows variable of the same name",
          parameter
        )?;
      }
      RequiredParameterFollowsDefaultParameter { parameter } => {
        writeln!(
          f,
          "Non-default parameter `{}` follows default parameter",
          parameter
        )?;
      }
      ParameterFollowsVariadicParameter { parameter } => {
        writeln!(f, "Parameter `{}` follows variadic parameter", parameter)?;
      }
      MixedLeadingWhitespace { whitespace } => {
        writeln!(
          f,
          "Found a mix of tabs and spaces in leading whitespace: `{}`\n\
           Leading whitespace may consist of tabs or spaces, but not both",
          ShowWhitespace(whitespace)
        )?;
      }
      ExtraLeadingWhitespace => {
        writeln!(f, "Recipe line has extra leading whitespace")?;
      }
      FunctionArgumentCountMismatch {
        function,
        found,
        expected,
      } => {
        writeln!(
          f,
          "Function `{}` called with {} {} but takes {}",
          function,
          found,
          Count("argument", found),
          expected
        )?;
      }
      InconsistentLeadingWhitespace { expected, found } => {
        writeln!(
          f,
          "Recipe line has inconsistent leading whitespace. \
           Recipe started with `{}` but found line with `{}`",
          ShowWhitespace(expected),
          ShowWhitespace(found)
        )?;
      }
      UnknownAliasTarget { alias, target } => {
        writeln!(f, "Alias `{}` has an unknown target `{}`", alias, target)?;
      }
      UnknownDependency { recipe, unknown } => {
        writeln!(
          f,
          "Recipe `{}` has unknown dependency `{}`",
          recipe, unknown
        )?;
      }
      UndefinedVariable { variable } => {
        writeln!(f, "Variable `{}` not defined", variable)?;
      }
      UnknownFunction { function } => {
        writeln!(f, "Call to unknown function `{}`", function)?;
      }
      UnknownStartOfToken => {
        writeln!(f, "Unknown start of token:")?;
      }
      UnpairedCarriageReturn => {
        writeln!(f, "Unpaired carriage return")?;
      }
      UnterminatedInterpolation => {
        writeln!(f, "Unterminated interpolation")?;
      }
      UnterminatedString => {
        writeln!(f, "Unterminated string")?;
      }
      UnterminatedBacktick => {
        writeln!(f, "Unterminated backtick")?;
      }
      Internal { ref message } => {
        writeln!(
          f,
          "Internal error, this may indicate a bug in just: {}\n\
           consider filing an issue: https://github.com/casey/just/issues/new",
          message
        )?;
      }
    }

    write!(f, "{}", message.suffix())?;

    write_message_context(
      f,
      Color::fmt(f).error(),
      self.src,
      self.offset,
      self.line,
      self.column,
      self.width,
    )
  }
}
