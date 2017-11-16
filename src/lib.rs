#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate tempdir;
extern crate itertools;
extern crate ansi_term;
extern crate unicode_width;
extern crate edit_distance;
extern crate libc;
extern crate brev;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod unit;

#[cfg(test)]
mod integration;

#[cfg(test)]
mod search;

mod platform;
mod app;
mod color;
mod compilation_error;
mod runtime_error;
mod formatting;
mod justfile;
mod recipe;
mod token;
mod parser;
mod tokenizer;
mod cooked_string;

use compilation_error::{CompilationError, CompilationErrorKind};
use runtime_error::RuntimeError;
use justfile::Justfile;
use recipe::Recipe;
use token::{Token, TokenKind};
use parser::Parser;
use cooked_string::CookedString;

use tokenizer::tokenize;

mod prelude {
  pub use libc::{EXIT_FAILURE, EXIT_SUCCESS};
  pub use regex::Regex;
  pub use std::io::prelude::*;
  pub use std::path::{Path, PathBuf};
  pub use std::{cmp, env, fs, fmt, io, iter, process};
  pub use std::collections::{BTreeMap as Map, BTreeSet as Set};
  pub use std::fmt::Display;
  pub use std::borrow::Cow;

  pub fn default<T: Default>() -> T {
    Default::default()
  }

  pub fn empty<T, C: iter::FromIterator<T>>() -> C {
    iter::empty().collect()
  }

  pub use std::ops::Range;

  pub fn contains<T: PartialOrd + Copy>(range: &Range<T>,  i: T) -> bool {
    i >= range.start && i < range.end
  }

  pub fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
  }
}

use prelude::*;

pub use app::app;

use brev::output;
use color::Color;
use std::fmt::Display;

const DEFAULT_SHELL: &'static str = "sh";

trait Slurp {
  fn slurp(&mut self) -> Result<String, std::io::Error>;
}

impl Slurp for fs::File {
  fn slurp(&mut self) -> Result<String, std::io::Error> {
    let mut destination = String::new();
    self.read_to_string(&mut destination)?;
    Ok(destination)
  }
}

/// Split a shebang line into a command and an optional argument
fn split_shebang(shebang: &str) -> Option<(&str, Option<&str>)> {
  lazy_static! {
    static ref EMPTY:    Regex = re(r"^#!\s*$");
    static ref SIMPLE:   Regex = re(r"^#!(\S+)\s*$");
    static ref ARGUMENT: Regex = re(r"^#!(\S+)\s+(\S.*?)?\s*$");
  }

  if EMPTY.is_match(shebang) {
    Some(("", None))
  } else if let Some(captures) = SIMPLE.captures(shebang) {
    Some((captures.get(1).unwrap().as_str(), None))
  } else if let Some(captures) = ARGUMENT.captures(shebang) {
    Some((captures.get(1).unwrap().as_str(), Some(captures.get(2).unwrap().as_str())))
  } else {
    None
  }
}

#[derive(PartialEq, Debug)]
pub struct Parameter<'a> {
  default:  Option<String>,
  name:     &'a str,
  token:    Token<'a>,
  variadic: bool,
}

impl<'a> Display for Parameter<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let color = Color::fmt(f);
    if self.variadic {
      write!(f, "{}", color.annotation().paint("+"))?;
    }
    write!(f, "{}", color.parameter().paint(self.name))?;
    if let Some(ref default) = self.default {
      let escaped = default.chars().flat_map(char::escape_default).collect::<String>();;
      write!(f, r#"='{}'"#, color.string().paint(&escaped))?;
    }
    Ok(())
  }
}

#[derive(PartialEq, Debug)]
pub enum Fragment<'a> {
  Text{text: Token<'a>},
  Expression{expression: Expression<'a>},
}

impl<'a> Fragment<'a> {
  fn continuation(&self) -> bool {
    match *self {
      Fragment::Text{ref text} => text.lexeme.ends_with('\\'),
      _ => false,
    }
  }
}

#[derive(PartialEq, Debug)]
pub enum Expression<'a> {
  Variable{name: &'a str, token: Token<'a>},
  String{cooked_string: CookedString<'a>},
  Backtick{raw: &'a str, token: Token<'a>},
  Concatination{lhs: Box<Expression<'a>>, rhs: Box<Expression<'a>>},
}

impl<'a> Expression<'a> {
  fn variables(&'a self) -> Variables<'a> {
    Variables {
      stack: vec![self],
    }
  }
}

struct Variables<'a> {
  stack: Vec<&'a Expression<'a>>,
}

impl<'a> Iterator for Variables<'a> {
  type Item = &'a Token<'a>;

  fn next(&mut self) -> Option<&'a Token<'a>> {
    match self.stack.pop() {
      None | Some(&Expression::String{..}) | Some(&Expression::Backtick{..}) => None,
      Some(&Expression::Variable{ref token,..})          => Some(token),
      Some(&Expression::Concatination{ref lhs, ref rhs}) => {
        self.stack.push(lhs);
        self.stack.push(rhs);
        self.next()
      }
    }
  }
}

impl<'a> Display for Expression<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Expression::Backtick     {raw, ..          } => write!(f, "`{}`", raw)?,
      Expression::Concatination{ref lhs, ref rhs } => write!(f, "{} + {}", lhs, rhs)?,
      Expression::String       {ref cooked_string} => write!(f, "\"{}\"", cooked_string.raw)?,
      Expression::Variable     {name, ..         } => write!(f, "{}", name)?,
    }
    Ok(())
  }
}

fn export_env<'a>(
  command: &mut process::Command,
  scope:   &Map<&'a str, String>,
  exports: &Set<&'a str>,
) -> Result<(), RuntimeError<'a>> {
  for name in exports {
    if let Some(value) = scope.get(name) {
      command.env(name, value);
    } else {
      return Err(RuntimeError::InternalError {
        message: format!("scope does not contain exported variable `{}`",  name),
      });
    }
  }
  Ok(())
}

fn run_backtick<'a>(
  raw:     &str,
  token:   &Token<'a>,
  scope:   &Map<&'a str, String>,
  exports: &Set<&'a str>,
  quiet:   bool,
) -> Result<String, RuntimeError<'a>> {
  let mut cmd = process::Command::new(DEFAULT_SHELL);

  export_env(&mut cmd, scope, exports)?;

  cmd.arg("-cu")
     .arg(raw);

  cmd.stderr(if quiet {
    process::Stdio::null()
  } else {
    process::Stdio::inherit()
  });

  output(cmd).map_err(|output_error| RuntimeError::Backtick{token: token.clone(), output_error})
}

fn resolve_recipes<'a>(
  recipes:     &Map<&'a str, Recipe<'a>>,
  assignments: &Map<&'a str, Expression<'a>>,
  text:        &'a str,
) -> Result<(), CompilationError<'a>> {
  let mut resolver = Resolver {
    seen:              empty(),
    stack:             empty(),
    resolved:          empty(),
    recipes:           recipes,
  };

  for recipe in recipes.values() {
    resolver.resolve(recipe)?;
    resolver.seen = empty();
  }

  for recipe in recipes.values() {
    for line in &recipe.lines {
      for fragment in line {
        if let Fragment::Expression{ref expression, ..} = *fragment {
          for variable in expression.variables() {
            let name = variable.lexeme;
            let undefined = !assignments.contains_key(name)
              && !recipe.parameters.iter().any(|p| p.name == name);
            if undefined {
              // There's a borrow issue here that seems too difficult to solve.
              // The error derived from the variable token has too short a lifetime,
              // so we create a new error from its contents, which do live long
              // enough.
              //
              // I suspect the solution here is to give recipes, pieces, and expressions
              // two lifetime parameters instead of one, with one being the lifetime
              // of the struct, and the second being the lifetime of the tokens
              // that it contains
              let error = variable.error(CompilationErrorKind::UndefinedVariable{variable: name});
              return Err(CompilationError {
                text:   text,
                index:  error.index,
                line:   error.line,
                column: error.column,
                width:  error.width,
                kind:   CompilationErrorKind::UndefinedVariable {
                  variable: &text[error.index..error.index + error.width.unwrap()],
                }
              });
            }
          }
        }
      }
    }
  }

  Ok(())
}

struct Resolver<'a: 'b, 'b> {
  stack:    Vec<&'a str>,
  seen:     Set<&'a str>,
  resolved: Set<&'a str>,
  recipes:  &'b Map<&'a str, Recipe<'a>>,
}

impl<'a, 'b> Resolver<'a, 'b> {
  fn resolve(&mut self, recipe: &Recipe<'a>) -> Result<(), CompilationError<'a>> {
    if self.resolved.contains(recipe.name) {
      return Ok(())
    }
    self.stack.push(recipe.name);
    self.seen.insert(recipe.name);
    for dependency_token in &recipe.dependency_tokens {
      match self.recipes.get(dependency_token.lexeme) {
        Some(dependency) => if !self.resolved.contains(dependency.name) {
          if self.seen.contains(dependency.name) {
            let first = self.stack[0];
            self.stack.push(first);
            return Err(dependency_token.error(CompilationErrorKind::CircularRecipeDependency {
              recipe: recipe.name,
              circle: self.stack.iter()
                .skip_while(|name| **name != dependency.name)
                .cloned().collect()
            }));
          }
          self.resolve(dependency)?;
        },
        None => return Err(dependency_token.error(CompilationErrorKind::UnknownDependency {
          recipe:  recipe.name,
          unknown: dependency_token.lexeme
        })),
      }
    }
    self.resolved.insert(recipe.name);
    self.stack.pop();
    Ok(())
  }
}

fn resolve_assignments<'a>(
  assignments:       &Map<&'a str, Expression<'a>>,
  assignment_tokens: &Map<&'a str, Token<'a>>,
) -> Result<(), CompilationError<'a>> {

  let mut resolver = AssignmentResolver {
    assignments:       assignments,
    assignment_tokens: assignment_tokens,
    stack:             empty(),
    seen:              empty(),
    evaluated:         empty(),
  };

  for name in assignments.keys() {
    resolver.resolve_assignment(name)?;
  }

  Ok(())
}

struct AssignmentResolver<'a: 'b, 'b> {
  assignments:       &'b Map<&'a str, Expression<'a>>,
  assignment_tokens: &'b Map<&'a str, Token<'a>>,
  stack:             Vec<&'a str>,
  seen:              Set<&'a str>,
  evaluated:         Set<&'a str>,
}

impl<'a: 'b, 'b> AssignmentResolver<'a, 'b> {
  fn resolve_assignment(&mut self, name: &'a str) -> Result<(), CompilationError<'a>> {
    if self.evaluated.contains(name) {
      return Ok(());
    }

    self.seen.insert(name);
    self.stack.push(name);

    if let Some(expression) = self.assignments.get(name) {
      self.resolve_expression(expression)?;
      self.evaluated.insert(name);
    } else {
      return Err(internal_error(format!("attempted to resolve unknown assignment `{}`", name)));
    }
    Ok(())
  }

  fn resolve_expression(&mut self, expression: &Expression<'a>) -> Result<(), CompilationError<'a>> {
    match *expression {
      Expression::Variable{name, ref token} => {
        if self.evaluated.contains(name) {
          return Ok(());
        } else if self.seen.contains(name) {
          let token = &self.assignment_tokens[name];
          self.stack.push(name);
          return Err(token.error(CompilationErrorKind::CircularVariableDependency {
            variable: name,
            circle:   self.stack.clone(),
          }));
        } else if self.assignments.contains_key(name) {
          self.resolve_assignment(name)?;
        } else {
          return Err(token.error(CompilationErrorKind::UndefinedVariable{variable: name}));
        }
      }
      Expression::Concatination{ref lhs, ref rhs} => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)?;
      }
      Expression::String{..} | Expression::Backtick{..} => {}
    }
    Ok(())
  }
}

fn evaluate_assignments<'a>(
  assignments: &Map<&'a str, Expression<'a>>,
  overrides:   &Map<&str, &str>,
  quiet:       bool,
) -> Result<Map<&'a str, String>, RuntimeError<'a>> {
  let mut evaluator = Evaluator {
    assignments: assignments,
    evaluated:   empty(),
    exports:     &empty(),
    overrides:   overrides,
    quiet:       quiet,
    scope:       &empty(),
  };

  for name in assignments.keys() {
    evaluator.evaluate_assignment(name)?;
  }

  Ok(evaluator.evaluated)
}

struct Evaluator<'a: 'b, 'b> {
  assignments: &'b Map<&'a str, Expression<'a>>,
  evaluated:   Map<&'a str, String>,
  exports:     &'b Set<&'a str>,
  overrides:   &'b Map<&'b str, &'b str>,
  quiet:       bool,
  scope:       &'b Map<&'a str, String>,
}

impl<'a, 'b> Evaluator<'a, 'b> {
  fn evaluate_line(
    &mut self,
    line:      &[Fragment<'a>],
    arguments: &Map<&str, Cow<str>>
  ) -> Result<String, RuntimeError<'a>> {
    let mut evaluated = String::new();
    for fragment in line {
      match *fragment {
        Fragment::Text{ref text} => evaluated += text.lexeme,
        Fragment::Expression{ref expression} => {
          evaluated += &self.evaluate_expression(expression, arguments)?;
        }
      }
    }
    Ok(evaluated)
  }

  fn evaluate_assignment(&mut self, name: &'a str) -> Result<(), RuntimeError<'a>> {
    if self.evaluated.contains_key(name) {
      return Ok(());
    }

    if let Some(expression) = self.assignments.get(name) {
      if let Some(value) = self.overrides.get(name) {
        self.evaluated.insert(name, value.to_string());
      } else {
        let value = self.evaluate_expression(expression, &empty())?;
        self.evaluated.insert(name, value);
      }
    } else {
      return Err(RuntimeError::InternalError {
        message: format!("attempted to evaluated unknown assignment {}", name)
      });
    }

    Ok(())
  }

  fn evaluate_expression(
    &mut self,
    expression: &Expression<'a>,
    arguments: &Map<&str, Cow<str>>
  ) -> Result<String, RuntimeError<'a>> {
    Ok(match *expression {
      Expression::Variable{name, ..} => {
        if self.evaluated.contains_key(name) {
          self.evaluated[name].clone()
        } else if self.scope.contains_key(name) {
          self.scope[name].clone()
        } else if self.assignments.contains_key(name) {
          self.evaluate_assignment(name)?;
          self.evaluated[name].clone()
        } else if arguments.contains_key(name) {
          arguments[name].to_string()
        } else {
          return Err(RuntimeError::InternalError {
            message: format!("attempted to evaluate undefined variable `{}`", name)
          });
        }
      }
      Expression::String{ref cooked_string} => cooked_string.cooked.clone(),
      Expression::Backtick{raw, ref token} => {
        run_backtick(raw, token, self.scope, self.exports, self.quiet)?
      }
      Expression::Concatination{ref lhs, ref rhs} => {
        self.evaluate_expression(lhs, arguments)?
          +
        &self.evaluate_expression(rhs, arguments)?
      }
    })
  }
}

fn internal_error(message: String) -> CompilationError<'static> {
  CompilationError {
    text:   "",
    index:  0,
    line:   0,
    column: 0,
    width:  None,
    kind:   CompilationErrorKind::InternalError { message: message }
  }
}

#[derive(Default)]
pub struct RunOptions<'a> {
  dry_run:   bool,
  evaluate:  bool,
  highlight: bool,
  overrides: Map<&'a str, &'a str>,
  quiet:     bool,
  shell:     Option<&'a str>,
  color:     Color,
  verbose:   bool,
}

fn compile(text: &str) -> Result<Justfile, CompilationError> {
  let tokens = tokenize(text)?;
  let parser = Parser::new(text, tokens);
  parser.justfile()
}
