#[macro_use]
extern crate lazy_static;
extern crate ansi_term;
extern crate brev;
extern crate clap;
extern crate dotenv;
extern crate edit_distance;
extern crate itertools;
extern crate libc;
extern crate regex;
extern crate target;
extern crate tempdir;
extern crate unicode_width;

#[cfg(test)]
#[macro_use]
mod testing;

mod assignment_evaluator;
mod assignment_resolver;
mod color;
mod command_ext;
mod compilation_error;
mod configuration;
mod cooked_string;
mod load_dotenv;
mod expression;
mod fragment;
mod functions;
mod justfile;
mod lexer;
mod misc;
mod parameter;
mod parser;
mod platform;
mod range_ext;
mod recipe;
mod recipe_resolver;
mod run;
mod runtime_error;
mod shebang;
mod token;

mod common {
  pub use std::borrow::Cow;
  pub use std::collections::{BTreeMap as Map, BTreeSet as Set};
  pub use std::fmt::Display;
  pub use std::io::prelude::*;
  pub use std::ops::Range;
  pub use std::path::{Path, PathBuf};
  pub use std::process::Command;
  pub use std::{cmp, env, fs, fmt, io, iter, process, vec, usize};

  pub use color::Color;
  pub use libc::{EXIT_FAILURE, EXIT_SUCCESS};
  pub use regex::Regex;
  pub use tempdir::TempDir;

  pub use assignment_evaluator::AssignmentEvaluator;
  pub use assignment_resolver::AssignmentResolver;
  pub use command_ext::CommandExt;
  pub use compilation_error::{CompilationError, CompilationErrorKind, CompilationResult};
  pub use configuration::Configuration;
  pub use cooked_string::CookedString;
  pub use expression::Expression;
  pub use fragment::Fragment;
  pub use justfile::Justfile;
  pub use lexer::Lexer;
  pub use load_dotenv::load_dotenv;
  pub use misc::{default, empty};
  pub use parameter::Parameter;
  pub use parser::Parser;
  pub use range_ext::RangeExt;
  pub use recipe::Recipe;
  pub use recipe_resolver::RecipeResolver;
  pub use runtime_error::{RuntimeError, RunResult};
  pub use shebang::Shebang;
  pub use token::{Token, TokenKind};
}

use common::*;

fn main() {
  run::run();
}
