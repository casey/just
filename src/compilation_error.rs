use common::*;

use misc::{Or, write_error_context, show_whitespace};

#[derive(Debug, PartialEq)]
pub struct CompilationError<'a> {
  pub text:   &'a str,
  pub index:  usize,
  pub line:   usize,
  pub column: usize,
  pub width:  Option<usize>,
  pub kind:   CompilationErrorKind<'a>,
}

#[derive(Debug, PartialEq)]
pub enum CompilationErrorKind<'a> {
  CircularRecipeDependency{recipe: &'a str, circle: Vec<&'a str>},
  CircularVariableDependency{variable: &'a str, circle: Vec<&'a str>},
  DependencyHasParameters{recipe: &'a str, dependency: &'a str},
  DuplicateDependency{recipe: &'a str, dependency: &'a str},
  DuplicateParameter{recipe: &'a str, parameter: &'a str},
  DuplicateRecipe{recipe: &'a str, first: usize},
  DuplicateVariable{variable: &'a str},
  ExtraLeadingWhitespace,
  InconsistentLeadingWhitespace{expected: &'a str, found: &'a str},
  InternalError{message: String},
  InvalidEscapeSequence{character: char},
  MixedLeadingWhitespace{whitespace: &'a str},
  OuterShebang,
  ParameterShadowsVariable{parameter: &'a str},
  RequiredParameterFollowsDefaultParameter{parameter: &'a str},
  ParameterFollowsVariadicParameter{parameter: &'a str},
  UndefinedVariable{variable: &'a str},
  UnexpectedToken{expected: Vec<TokenKind>, found: TokenKind},
  UnknownDependency{recipe: &'a str, unknown: &'a str},
  UnknownStartOfToken,
  UnterminatedString,
}

impl<'a> Display for CompilationError<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    use CompilationErrorKind::*;
    let error   = Color::fmt(f).error();
    let message = Color::fmt(f).message();

    write!(f, "{} {}", error.paint("error:"), message.prefix())?;

    match self.kind {
      CircularRecipeDependency{recipe, ref circle} => {
        if circle.len() == 2 {
          writeln!(f, "Recipe `{}` depends on itself", recipe)?;
        } else {
          writeln!(f, "Recipe `{}` has circular dependency `{}`",
                      recipe, circle.join(" -> "))?;
        }
      }
      CircularVariableDependency{variable, ref circle} => {
        if circle.len() == 2 {
          writeln!(f, "Variable `{}` is defined in terms of itself", variable)?;
        } else {
          writeln!(f, "Variable `{}` depends on its own value: `{}`",
                      variable, circle.join(" -> "))?;
        }
      }
      InvalidEscapeSequence{character} => {
        writeln!(f, "`\\{}` is not a valid escape sequence",
                    character.escape_default().collect::<String>())?;
      }
      DuplicateParameter{recipe, parameter} => {
        writeln!(f, "Recipe `{}` has duplicate parameter `{}`", recipe, parameter)?;
      }
      DuplicateVariable{variable} => {
        writeln!(f, "Variable `{}` has multiple definitions", variable)?;
      }
      UnexpectedToken{ref expected, found} => {
        writeln!(f, "Expected {}, but found {}", Or(expected), found)?;
      }
      DuplicateDependency{recipe, dependency} => {
        writeln!(f, "Recipe `{}` has duplicate dependency `{}`", recipe, dependency)?;
      }
      DuplicateRecipe{recipe, first} => {
        writeln!(f, "Recipe `{}` first defined on line {} is redefined on line {}",
                    recipe, first + 1, self.line + 1)?;
      }
      DependencyHasParameters{recipe, dependency} => {
        writeln!(f, "Recipe `{}` depends on `{}` which requires arguments. \
                    Dependencies may not require arguments", recipe, dependency)?;
      }
      ParameterShadowsVariable{parameter} => {
        writeln!(f, "Parameter `{}` shadows variable of the same name", parameter)?;
      }
      RequiredParameterFollowsDefaultParameter{parameter} => {
        writeln!(f, "Non-default parameter `{}` follows default parameter", parameter)?;
      }
      ParameterFollowsVariadicParameter{parameter} => {
        writeln!(f, "Parameter `{}` follows variadic parameter", parameter)?;
      }
      MixedLeadingWhitespace{whitespace} => {
        writeln!(f,
          "Found a mix of tabs and spaces in leading whitespace: `{}`\n\
          Leading whitespace may consist of tabs or spaces, but not both",
          show_whitespace(whitespace)
        )?;
      }
      ExtraLeadingWhitespace => {
        writeln!(f, "Recipe line has extra leading whitespace")?;
      }
      InconsistentLeadingWhitespace{expected, found} => {
        writeln!(f,
          "Recipe line has inconsistent leading whitespace. \
           Recipe started with `{}` but found line with `{}`",
          show_whitespace(expected), show_whitespace(found)
        )?;
      }
      OuterShebang => {
        writeln!(f, "`#!` is reserved syntax outside of recipes")?;
      }
      UnknownDependency{recipe, unknown} => {
        writeln!(f, "Recipe `{}` has unknown dependency `{}`", recipe, unknown)?;
      }
      UndefinedVariable{variable} => {
        writeln!(f, "Variable `{}` not defined", variable)?;
      }
      UnknownStartOfToken => {
        writeln!(f, "Unknown start of token:")?;
      }
      UnterminatedString => {
        writeln!(f, "Unterminated string")?;
      }
      InternalError{ref message} => {
        writeln!(f, "Internal error, this may indicate a bug in just: {}\n\
                     consider filing an issue: https://github.com/casey/just/issues/new",
                     message)?;
      }
    }

    write!(f, "{}", message.suffix())?;

    write_error_context(f, self.text, self.index, self.line, self.column, self.width)
  }
}
