pub(crate) use std::{
  borrow::Cow,
  cmp,
  collections::{BTreeMap, BTreeSet},
  env,
  fmt::{self, Display, Formatter},
  fs, io, iter,
  ops::{Range, RangeInclusive},
  path::{Path, PathBuf},
  process,
  process::Command,
  str::Chars,
  sync::{Mutex, MutexGuard},
  usize, vec,
};

pub(crate) use edit_distance::edit_distance;
pub(crate) use libc::{EXIT_FAILURE, EXIT_SUCCESS};
pub(crate) use log::warn;
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
  expression::Expression,
  fragment::Fragment,
  function::{evaluate_function, resolve_function},
  function_context::FunctionContext,
  functions::Functions,
  interrupt_guard::InterruptGuard,
  interrupt_handler::InterruptHandler,
  justfile::Justfile,
  lexer::Lexer,
  load_dotenv::load_dotenv,
  misc::{default, empty},
  parameter::Parameter,
  parser::Parser,
  position::Position,
  recipe::Recipe,
  recipe_context::RecipeContext,
  recipe_resolver::RecipeResolver,
  runtime_error::{RunResult, RuntimeError},
  shebang::Shebang,
  state::State,
  string_literal::StringLiteral,
  token::Token,
  token_kind::TokenKind,
  use_color::UseColor,
  variables::Variables,
  verbosity::Verbosity,
};

#[allow(unused_imports)]
pub(crate) use std::io::prelude::*;

#[allow(unused_imports)]
pub(crate) use crate::command_ext::CommandExt;

#[allow(unused_imports)]
pub(crate) use crate::range_ext::RangeExt;
