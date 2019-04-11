pub(crate) use std::{
  borrow::Cow,
  cmp,
  collections::{BTreeMap as Map, BTreeSet as Set},
  env,
  fmt::{self, Display, Formatter},
  fs, io, iter,
  ops::{Range, RangeInclusive},
  path::{Path, PathBuf},
  process,
  process::Command,
  sync::{Mutex, MutexGuard},
  usize, vec,
};

pub(crate) use edit_distance::edit_distance;
pub(crate) use libc::{EXIT_FAILURE, EXIT_SUCCESS};
pub(crate) use log::warn;
pub(crate) use regex::Regex;
pub(crate) use tempdir::TempDir;
pub(crate) use unicode_width::UnicodeWidthChar;

pub(crate) use crate::{
  alias::Alias,
  alias_resolver::AliasResolver,
  assignment_evaluator::AssignmentEvaluator,
  assignment_resolver::AssignmentResolver,
  color::Color,
  compilation_error::{CompilationError, CompilationErrorKind, CompilationResult},
  configuration::Configuration,
  cooked_string::CookedString,
  expression::Expression,
  fragment::Fragment,
  function::{evaluate_function, resolve_function, FunctionContext},
  interrupt_handler::InterruptHandler,
  justfile::Justfile,
  lexer::Lexer,
  load_dotenv::load_dotenv,
  misc::{default, empty},
  parameter::Parameter,
  parser::Parser,
  recipe::{Recipe, RecipeContext},
  recipe_resolver::RecipeResolver,
  runtime_error::{RunResult, RuntimeError},
  shebang::Shebang,
  token::{Token, TokenKind},
  verbosity::Verbosity,
};

#[allow(unused_imports)]
pub(crate) use std::io::prelude::*;

#[allow(unused_imports)]
pub(crate) use crate::command_ext::CommandExt;

#[allow(unused_imports)]
pub(crate) use crate::range_ext::RangeExt;
