// stdlib
pub(crate) use std::{
  borrow::Cow,
  cmp,
  collections::{BTreeMap, BTreeSet},
  env,
  ffi::OsString,
  fmt::{self, Debug, Display, Formatter},
  fs,
  io::{self, Cursor, Write},
  iter::{self, FromIterator},
  ops::{Index, Range, RangeInclusive},
  path::{Path, PathBuf},
  process::{self, Command, Stdio},
  rc::Rc,
  str::{self, Chars},
  sync::{Mutex, MutexGuard},
  usize, vec,
};

// dependencies
pub(crate) use derivative::Derivative;
pub(crate) use edit_distance::edit_distance;
pub(crate) use libc::EXIT_FAILURE;
pub(crate) use log::{info, warn};
pub(crate) use snafu::{ResultExt, Snafu};
pub(crate) use strum::{Display, EnumString, IntoStaticStr};
pub(crate) use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// modules
pub(crate) use crate::{config_error, setting};

// functions
pub(crate) use crate::{default::default, empty::empty, load_dotenv::load_dotenv, output::output};

// traits
pub(crate) use crate::{
  command_ext::CommandExt, error::Error, error_result_ext::ErrorResultExt, keyed::Keyed,
  ordinal::Ordinal, platform_interface::PlatformInterface, range_ext::RangeExt,
};

// structs and enums
pub(crate) use crate::{
  alias::Alias, analyzer::Analyzer, assignment::Assignment,
  assignment_resolver::AssignmentResolver, binding::Binding, color::Color,
  compilation_error::CompilationError, compilation_error_kind::CompilationErrorKind,
  compiler::Compiler, config::Config, config_error::ConfigError, count::Count,
  delimiter::Delimiter, dependency::Dependency, enclosure::Enclosure, evaluator::Evaluator,
  expression::Expression, fragment::Fragment, function::Function,
  function_context::FunctionContext, interrupt_guard::InterruptGuard,
  interrupt_handler::InterruptHandler, item::Item, justfile::Justfile, keyword::Keyword,
  lexer::Lexer, line::Line, list::List, load_error::LoadError, module::Module, name::Name,
  output_error::OutputError, parameter::Parameter, parameter_kind::ParameterKind, parser::Parser,
  platform::Platform, position::Position, positional::Positional, recipe::Recipe,
  recipe_context::RecipeContext, recipe_resolver::RecipeResolver, runtime_error::RuntimeError,
  scope::Scope, search::Search, search_config::SearchConfig, search_error::SearchError, set::Set,
  setting::Setting, settings::Settings, shebang::Shebang, show_whitespace::ShowWhitespace,
  string_kind::StringKind, string_literal::StringLiteral, subcommand::Subcommand,
  suggestion::Suggestion, table::Table, thunk::Thunk, token::Token, token_kind::TokenKind,
  unresolved_dependency::UnresolvedDependency, unresolved_recipe::UnresolvedRecipe,
  use_color::UseColor, variables::Variables, verbosity::Verbosity, warning::Warning,
};

// type aliases
pub(crate) type CompilationResult<'a, T> = Result<T, CompilationError<'a>>;
pub(crate) type ConfigResult<T> = Result<T, ConfigError>;
pub(crate) type RunResult<'a, T> = Result<T, RuntimeError<'a>>;
pub(crate) type SearchResult<T> = Result<T, SearchError>;

// modules used in tests
#[cfg(test)]
pub(crate) use crate::testing;

// structs and enums used in tests
#[cfg(test)]
pub(crate) use crate::{node::Node, tree::Tree};
