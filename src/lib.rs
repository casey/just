#![deny(clippy::all, clippy::pedantic)]
#![allow(
  clippy::doc_markdown,
  clippy::empty_enum,
  clippy::enum_glob_use,
  clippy::if_not_else,
  clippy::missing_errors_doc,
  clippy::needless_lifetimes,
  clippy::needless_pass_by_value,
  clippy::non_ascii_literal,
  clippy::shadow_unrelated,
  clippy::struct_excessive_bools,
  clippy::too_many_lines,
  clippy::wildcard_imports
)]

pub use crate::run::run;

// Used in integration tests.
#[doc(hidden)]
pub use unindent::unindent;

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

// Used by Janus, https://github.com/casey/janus, a tool
// that analyses all public justfiles on GitHub to avoid
// breaking changes.
#[doc(hidden)]
pub mod summary;

mod alias;
mod analyzer;
mod assignment;
mod assignment_resolver;
mod ast;
mod binding;
mod color;
mod color_display;
mod command_ext;
mod common;
mod compile_error;
mod compile_error_kind;
mod compiler;
mod completions;
mod conditional_operator;
mod config;
mod config_error;
mod count;
mod delimiter;
mod dependency;
mod dump_format;
mod enclosure;
mod error;
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
mod loader;
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
