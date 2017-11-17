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
mod command_ext;
mod range_ext;

#[cfg(test)] mod testing;

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
use command_ext::CommandExt;

use tokenizer::tokenize;

pub use app::app;

mod common {
  pub use libc::{EXIT_FAILURE, EXIT_SUCCESS};
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
}

fn compile(text: &str) -> Result<Justfile, CompilationError> {
  let tokens = tokenize(text)?;
  let parser = Parser::new(text, tokens);
  parser.justfile()
}

const DEFAULT_SHELL: &'static str = "sh";

