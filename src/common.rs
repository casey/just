// stdlib
pub(crate) use std::{
  borrow::Cow,
  cmp,
  collections::{BTreeMap, BTreeSet},
  convert::AsRef,
  env,
  ffi::OsStr,
  fmt::{self, Display, Formatter},
  fs, io, iter,
  ops::{Range, RangeInclusive},
  path::{Path, PathBuf},
  process::{self, Command},
  str::Chars,
  sync::{Mutex, MutexGuard},
  usize, vec,
};

// dependencies
pub(crate) use edit_distance::edit_distance;
pub(crate) use libc::EXIT_FAILURE;
pub(crate) use log::warn;
pub(crate) use unicode_width::UnicodeWidthChar;

// modules
pub(crate) use crate::search;

// modules used in tests
#[cfg(test)]
pub(crate) use crate::testing;

// functions
pub(crate) use crate::{
  load_dotenv::load_dotenv,
  misc::{default, empty},
  output::output,
};

// structs and enums
pub(crate) use crate::{
  alias::Alias, alias_resolver::AliasResolver, assignment_evaluator::AssignmentEvaluator,
  assignment_resolver::AssignmentResolver, color::Color, compilation_error::CompilationError,
  compilation_error_kind::CompilationErrorKind, config::Config, config_error::ConfigError,
  expression::Expression, fragment::Fragment, function::Function,
  function_context::FunctionContext, functions::Functions, interrupt_guard::InterruptGuard,
  interrupt_handler::InterruptHandler, justfile::Justfile, lexer::Lexer, output_error::OutputError,
  parameter::Parameter, parser::Parser, platform::Platform, position::Position, recipe::Recipe,
  recipe_context::RecipeContext, recipe_resolver::RecipeResolver, runtime_error::RuntimeError,
  search_error::SearchError, shebang::Shebang, state::State, string_literal::StringLiteral,
  subcommand::Subcommand, token::Token, token_kind::TokenKind, use_color::UseColor,
  variables::Variables, verbosity::Verbosity, warning::Warning,
};

pub(crate) type CompilationResult<'a, T> = Result<T, CompilationError<'a>>;

pub(crate) type RunResult<'a, T> = Result<T, RuntimeError<'a>>;

pub(crate) type ConfigResult<T> = Result<T, ConfigError>;

#[allow(unused_imports)]
pub(crate) use std::io::prelude::*;

#[allow(unused_imports)]
pub(crate) use crate::command_ext::CommandExt;

#[allow(unused_imports)]
pub(crate) use crate::range_ext::RangeExt;

#[allow(unused_imports)]
pub(crate) use crate::ordinal::Ordinal;

#[allow(unused_imports)]
pub(crate) use crate::platform_interface::PlatformInterface;
