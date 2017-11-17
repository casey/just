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
mod recipe_resolver;
mod assignment_resolver;
mod assignment_evaluator;
mod run_options;

use run_options::RunOptions;
use compilation_error::{CompilationError, CompilationErrorKind};
use runtime_error::RuntimeError;
use justfile::Justfile;
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

fn compile(text: &str) -> Result<Justfile, CompilationError> {
  let tokens = tokenize(text)?;
  let parser = Parser::new(text, tokens);
  parser.justfile()
}
