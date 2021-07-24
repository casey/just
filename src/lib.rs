#![deny(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(
  clippy::blanket_clippy_restriction_lints,
  clippy::comparison_chain,
  clippy::create_dir,
  clippy::default_numeric_fallback,
  clippy::else_if_without_else,
  clippy::enum_glob_use,
  clippy::exhaustive_enums,
  clippy::exhaustive_structs,
  clippy::expect_used,
  clippy::filter_map,
  clippy::if_not_else,
  clippy::implicit_return,
  clippy::indexing_slicing,
  clippy::integer_arithmetic,
  clippy::let_underscore_must_use,
  clippy::map_unwrap_or,
  clippy::match_same_arms,
  clippy::missing_docs_in_private_items,
  clippy::missing_errors_doc,
  clippy::missing_inline_in_public_items,
  clippy::needless_pass_by_value,
  clippy::non_ascii_literal,
  clippy::option_if_let_else,
  clippy::panic,
  clippy::panic_in_result_fn,
  clippy::pattern_type_mismatch,
  clippy::print_stdout,
  clippy::shadow_unrelated,
  clippy::string_add,
  clippy::struct_excessive_bools,
  clippy::too_many_lines,
  clippy::unnecessary_wraps,
  clippy::unreachable,
  clippy::unwrap_in_result,
  clippy::unwrap_used,
  clippy::use_debug,
  clippy::wildcard_enum_match_arm,
  clippy::wildcard_imports
)]

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
pub mod testing;

#[cfg(test)]
#[macro_use]
pub mod tree;

#[cfg(test)]
pub mod node;

#[cfg(fuzzing)]
pub(crate) mod fuzzing;

mod alias;
mod analyzer;
mod assignment;
mod assignment_resolver;
mod ast;
mod binding;
mod color;
mod command_ext;
mod common;
mod compilation_error;
mod compilation_error_kind;
mod compiler;
mod config;
mod config_error;
mod count;
mod delimiter;
mod dependency;
mod enclosure;
mod error;
mod error_result_ext;
mod evaluator;
mod expression;
mod fragment;
mod function;
mod function_context;
mod interrupt_guard;
mod interrupt_handler;
mod item;
mod justfile;
mod keyed;
mod keyword;
mod lexer;
mod line;
mod list;
mod load_dotenv;
mod load_error;
mod name;
mod ordinal;
mod output;
mod output_error;
mod parameter;
mod parameter_kind;
mod parser;
mod platform;
mod platform_interface;
mod position;
mod positional;
mod range_ext;
mod recipe;
mod recipe_context;
mod recipe_resolver;
mod run;
mod runtime_error;
mod scope;
mod search;
mod search_config;
mod search_error;
mod set;
mod setting;
mod settings;
mod shebang;
mod show_whitespace;
mod string_kind;
mod string_literal;
mod subcommand;
mod suggestion;
mod table;
mod thunk;
mod token;
mod token_kind;
mod unindent;
mod unresolved_dependency;
mod unresolved_recipe;
mod use_color;
mod variables;
mod verbosity;
mod warning;

pub use crate::run::run;

// Used in integration tests.
#[doc(hidden)]
pub use unindent::unindent;

// Used by Janus, https://github.com/casey/janus, a tool
// that analyses all public justfiles on GitHub to avoid
// breaking changes.
#[doc(hidden)]
pub mod summary;
