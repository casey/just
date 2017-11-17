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
mod configuration;
mod parameter;
mod expression;
mod fragment;
mod shebang;

use configuration::Configuration;
use compilation_error::{CompilationError, CompilationErrorKind};
use runtime_error::RuntimeError;
use justfile::Justfile;
use token::{Token, TokenKind};
use parser::Parser;
use cooked_string::CookedString;
use fragment::Fragment;
use expression::Expression;
use shebang::Shebang;

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

fn compile(text: &str) -> Result<Justfile, CompilationError> {
  let tokens = tokenize(text)?;
  let parser = Parser::new(text, tokens);
  parser.justfile()
}
