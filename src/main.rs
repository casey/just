#[macro_use]
extern crate lazy_static;
extern crate ansi_term;
extern crate brev;
extern crate clap;
extern crate edit_distance;
extern crate itertools;
extern crate libc;
extern crate regex;
extern crate tempdir;
extern crate unicode_width;

mod platform;
mod run;
mod color;
mod compilation_error;
mod runtime_error;
mod misc;
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

use tokenizer::tokenize;

mod common {
  pub use std::borrow::Cow;
  pub use std::collections::{BTreeMap as Map, BTreeSet as Set};
  pub use std::fmt::Display;
  pub use std::io::prelude::*;
  pub use std::path::{Path, PathBuf};
  pub use std::{cmp, env, fs, fmt, io, iter, process, vec};

  pub use color::Color;
  pub use libc::{EXIT_FAILURE, EXIT_SUCCESS};
  pub use regex::Regex;
  pub use tempdir::TempDir;

  pub use command_ext::CommandExt;
  pub use compilation_error::{CompilationError, CompilationErrorKind};
  pub use configuration::Configuration;
  pub use cooked_string::CookedString;
  pub use expression::Expression;
  pub use fragment::Fragment;
  pub use justfile::Justfile;
  pub use misc::{default, empty};
  pub use parameter::Parameter;
  pub use parser::Parser;
  pub use recipe::Recipe;
  pub use runtime_error::RuntimeError;
  pub use shebang::Shebang;
  pub use token::{Token, TokenKind};
}

use common::*;

fn compile(text: &str) -> Result<Justfile, CompilationError> {
  let tokens = tokenize(text)?;
  let parser = Parser::new(text, tokens);
  parser.justfile()
}

const DEFAULT_SHELL: &'static str = "sh";

fn main() {
  run::run();
}
