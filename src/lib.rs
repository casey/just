//! `just` is primarily used as a command-line binary, but does provide a
//! limited public library interface.
//!
//! Please keep in mind that there are no semantic version guarantees for the
//! library interface. It may break or change at any time.

pub(crate) use {
  crate::{
    alias::Alias,
    alias_style::AliasStyle,
    analyzer::Analyzer,
    arg_attribute::ArgAttribute,
    assignment::Assignment,
    assignment_resolver::AssignmentResolver,
    ast::Ast,
    attribute::{Attribute, AttributeKind},
    attribute_set::AttributeSet,
    binding::Binding,
    cache::Cache,
    cache_entry::CacheEntry,
    cache_key::CacheKey,
    cache_lock::CacheLock,
    cache_status::CacheStatus,
    color::Color,
    color_display::ColorDisplay,
    command_color::CommandColor,
    command_ext::CommandExt,
    compilation::Compilation,
    compile_error::CompileError,
    compile_error_kind::CompileErrorKind,
    compiler::Compiler,
    completer::Completer,
    conditional_operator::ConditionalOperator,
    config::Config,
    config_error::ConfigError,
    const_error::ConstError,
    const_eval_error::ConstEvalError,
    constants::constants,
    count::Count,
    delimiter::Delimiter,
    dependency::Dependency,
    dependency_argument::DependencyArgument,
    disabled::Disabled,
    dump_format::DumpFormat,
    element::Element,
    enclosure::Enclosure,
    environment::Environment,
    error::Error,
    evaluate_format::EvaluateFormat,
    evaluator::Evaluator,
    execution_context::ExecutionContext,
    executor::Executor,
    expression::Expression,
    format_string_part::FormatStringPart,
    fragment::Fragment,
    function::Function,
    function_definition::FunctionDefinition,
    indentation::Indentation,
    interpreter::Interpreter,
    invocation::Invocation,
    invocation_parser::InvocationParser,
    item::Item,
    justfile::Justfile,
    keyed::Keyed,
    keyword::Keyword,
    layer::Layer,
    lexer::Lexer,
    line::Line,
    list::List,
    list_feature::ListFeature,
    list_operator::ListOperator,
    load_dotenv::load_dotenv,
    loader::Loader,
    modulepath::Modulepath,
    name::Name,
    namepath::Namepath,
    number::Number,
    numerator::Numerator,
    ordinal::Ordinal,
    output_error::OutputError,
    parameter::{Bound, Parameter},
    parameter_kind::ParameterKind,
    parser::Parser,
    pattern::Pattern,
    platform::Platform,
    platform_interface::PlatformInterface,
    position::Position,
    positional::Positional,
    ran::Ran,
    range_ext::RangeExt,
    recipe::Recipe,
    recipe_resolver::RecipeResolver,
    recipe_signature::RecipeSignature,
    reference::Reference,
    references::References,
    request::Request,
    resolution::Resolution,
    scope::Scope,
    search::Search,
    search_config::SearchConfig,
    search_error::SearchError,
    set::Set,
    setting::Setting,
    settings::Settings,
    shebang::Shebang,
    shell::Shell,
    shell_kind::ShellKind,
    show_whitespace::ShowWhitespace,
    sigil::Sigil,
    signal::Signal,
    signal_handler::SignalHandler,
    source::Source,
    string_context::StringContext,
    string_delimiter::StringDelimiter,
    string_kind::StringKind,
    string_literal::StringLiteral,
    string_state::StringState,
    style::Style,
    subcommand::Subcommand,
    suggestion::Suggestion,
    switch::Switch,
    table::Table,
    tangle::tangle,
    times::Times,
    token::Token,
    token_kind::TokenKind,
    unresolved_dependency::UnresolvedDependency,
    unresolved_recipe::UnresolvedRecipe,
    unstable_feature::UnstableFeature,
    usage::Usage,
    use_color::UseColor,
    value::Value,
    verbosity::Verbosity,
    version::Version,
    warning::Warning,
    which::which,
  },
  camino::Utf8Path,
  clap::{CommandFactory, FromArgMatches, Parser as _, ValueEnum},
  clap_complete::{ArgValueCompleter, CompletionCandidate, PathCompleter, engine::ValueCompleter},
  lexiclean::Lexiclean,
  libc::EXIT_FAILURE,
  rand::seq::IndexedRandom,
  regex::Regex,
  serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    ser::{SerializeMap, SerializeSeq, SerializeStruct},
  },
  snafu::{ResultExt, Snafu},
  std::{
    borrow::{Borrow, Cow},
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, btree_map},
    env::{self, VarError},
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display, Formatter},
    fs::{self, File},
    io::{self, Write},
    iter::{self, FromIterator},
    mem,
    ops::Deref,
    ops::{Index, RangeInclusive},
    path::{self, Component, Path, PathBuf},
    process::{self, Command, ExitStatus, Stdio},
    slice,
    str::{self, Chars, FromStr},
    sync::{Arc, LazyLock, Mutex, MutexGuard},
    thread,
    time::Instant,
    vec,
  },
  strum::{Display, EnumDiscriminants, EnumIter, EnumString, IntoStaticStr},
  tempfile::TempDir,
  typed_arena::Arena,
  unicode_width::{UnicodeWidthChar, UnicodeWidthStr},
};

#[cfg(test)]
pub(crate) use crate::{node::Node, tree::Tree};

pub use crate::run::run;

#[doc(hidden)]
pub use {arguments::Arguments, request::Response, subcommand::INIT_JUSTFILE, unindent::unindent};

type CompileResult<'a, T = ()> = Result<T, CompileError<'a>>;
type ConfigResult<T> = Result<T, ConfigError>;
type RunResult<'a, T = ()> = Result<T, Error<'a>>;
type SearchResult<T> = Result<T, SearchError>;
type StringResult = Result<String, String>;
type ValueResult = Result<Value, String>;

type ModuleAlias<'src> = Alias<'src, Modulepath>;
type RecipeAlias<'src> = Alias<'src, Arc<Recipe<'src>>>;

const JUST_DIRECTORY: &str = "just";
const RECURSION_LIMIT: usize = if cfg!(windows) { 48 } else { 256 };
const TEMPDIR_PREFIX: &str = "just-";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[macro_use]
pub mod testing;

#[cfg(test)]
#[macro_use]
pub mod tree;

#[cfg(test)]
pub mod node;

#[cfg(fuzzing)]
pub mod fuzzing;

// Used for testing with the `--request` subcommand.
#[doc(hidden)]
pub mod request;

mod alias;
mod alias_style;
mod analyzer;
mod arg_attribute;
mod arguments;
mod assignment;
mod assignment_resolver;
mod ast;
mod attribute;
mod attribute_set;
mod binding;
mod cache;
mod cache_entry;
mod cache_key;
mod cache_lock;
mod cache_status;
mod color;
mod color_display;
mod command_color;
mod command_ext;
mod compilation;
mod compile_error;
mod compile_error_kind;
mod compiler;
mod completer;
mod conditional_operator;
mod config;
mod config_error;
mod const_error;
mod const_eval_error;
mod constants;
mod count;
mod delimiter;
mod dependency;
mod dependency_argument;
mod disabled;
mod dump_format;
mod element;
mod enclosure;
mod environment;
mod error;
mod evaluate_format;
mod evaluator;
mod execution_context;
mod executor;
mod expression;
mod filesystem;
mod format_string_part;
mod fragment;
mod function;
mod function_definition;
mod indentation;
mod interpreter;
mod invocation;
mod invocation_parser;
mod item;
mod justfile;
mod keyed;
mod keyword;
mod layer;
mod lexer;
mod line;
mod list;
mod list_feature;
mod list_operator;
mod load_dotenv;
mod loader;
mod modulepath;
mod name;
mod namepath;
mod number;
mod numerator;
mod ordinal;
mod output_error;
mod parameter;
mod parameter_kind;
mod parser;
mod pattern;
mod platform;
mod platform_interface;
mod position;
mod positional;
mod ran;
mod range_ext;
mod recipe;
mod recipe_resolver;
mod recipe_signature;
mod reference;
mod references;
mod resolution;
mod run;
mod scope;
mod search;
mod search_config;
mod search_error;
mod set;
mod setting;
mod settings;
mod shebang;
mod shell;
mod shell_kind;
mod show_whitespace;
mod sigil;
mod signal;
mod signal_handler;
#[cfg(unix)]
mod signals;
mod source;
mod string_context;
mod string_delimiter;
mod string_kind;
mod string_literal;
mod string_state;
mod style;
mod subcommand;
mod suggestion;
mod switch;
mod table;
mod tangle;
mod times;
mod token;
mod token_kind;
mod unindent;
mod unresolved_dependency;
mod unresolved_recipe;
mod unstable_feature;
mod usage;
mod use_color;
mod value;
mod verbosity;
mod version;
mod warning;
mod which;
