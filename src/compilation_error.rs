use common::*;

use misc::{maybe_s, show_whitespace, write_error_context, Or};

pub type CompilationResult<'a, T> = Result<T, CompilationError<'a>>;

#[derive(Debug, PartialEq)]
pub struct CompilationError<'a> {
  pub text: &'a str,
  pub index: usize,
  pub line: usize,
  pub column: usize,
  pub width: Option<usize>,
  pub kind: CompilationErrorKind<'a>,
}

#[derive(Debug, PartialEq)]
pub enum CompilationErrorKind<'a> {
  AliasShadowsRecipe {
    alias: &'a str,
    recipe_line: usize,
  },
  CircularRecipeDependency {
    recipe: &'a str,
    circle: Vec<&'a str>,
  },
  CircularVariableDependency {
    variable: &'a str,
    circle: Vec<&'a str>,
  },
  DependencyHasParameters {
    recipe: &'a str,
    dependency: &'a str,
  },
  DuplicateAlias {
    alias: &'a str,
    first: usize,
  },
  DuplicateDependency {
    recipe: &'a str,
    dependency: &'a str,
  },
  DuplicateParameter {
    recipe: &'a str,
    parameter: &'a str,
  },
  DuplicateRecipe {
    recipe: &'a str,
    first: usize,
  },
  DuplicateVariable {
    variable: &'a str,
  },
  ExtraLeadingWhitespace,
  FunctionArgumentCountMismatch {
    function: &'a str,
    found: usize,
    expected: usize,
  },
  InconsistentLeadingWhitespace {
    expected: &'a str,
    found: &'a str,
  },
  Internal {
    message: String,
  },
  InvalidEscapeSequence {
    character: char,
  },
  MixedLeadingWhitespace {
    whitespace: &'a str,
  },
  OuterShebang,
  ParameterFollowsVariadicParameter {
    parameter: &'a str,
  },
  ParameterShadowsVariable {
    parameter: &'a str,
  },
  RequiredParameterFollowsDefaultParameter {
    parameter: &'a str,
  },
  UndefinedVariable {
    variable: &'a str,
  },
  UnexpectedToken {
    expected: Vec<TokenKind>,
    found: TokenKind,
  },
  UnknownAliasTarget {
    alias: &'a str,
    target: &'a str,
  },
  UnknownDependency {
    recipe: &'a str,
    unknown: &'a str,
  },
  UnknownFunction {
    function: &'a str,
  },
  UnknownStartOfToken,
  UnterminatedInterpolation,
  UnterminatedString,
}

impl<'a> Display for CompilationError<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    use CompilationErrorKind::*;
    let error = Color::fmt(f).error();
    let message = Color::fmt(f).message();

    write!(f, "{} {}", error.paint("error:"), message.prefix())?;

    match self.kind {
      AliasShadowsRecipe { alias, recipe_line } => {
        writeln!(
          f,
          "Alias `{}` defined on `{}` shadows recipe defined on `{}`",
          alias,
          self.line + 1,
          recipe_line + 1,
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
        writeln!(f, "Expected {}, but found {}", Or(expected), found)?;
      }
      DuplicateAlias { alias, first } => {
        writeln!(
          f,
          "Alias `{}` first defined on line `{}` is redefined on line `{}`",
          alias,
          first + 1,
          self.line + 1,
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
          first + 1,
          self.line + 1
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
          show_whitespace(whitespace)
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
          "Function `{}` called with {} argument{} but takes {}",
          function,
          found,
          maybe_s(found),
          expected
        )?;
      }
      InconsistentLeadingWhitespace { expected, found } => {
        writeln!(
          f,
          "Recipe line has inconsistent leading whitespace. \
           Recipe started with `{}` but found line with `{}`",
          show_whitespace(expected),
          show_whitespace(found)
        )?;
      }
      OuterShebang => {
        writeln!(f, "`#!` is reserved syntax outside of recipes")?;
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
      UnterminatedInterpolation => {
        writeln!(f, "Unterminated interpolation")?;
      }
      UnterminatedString => {
        writeln!(f, "Unterminated string")?;
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

    write_error_context(f, self.text, self.index, self.line, self.column, self.width)
  }
}
