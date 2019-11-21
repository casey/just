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
mod alias_resolver;
mod analyzer;
mod assignment;
mod assignment_evaluator;
mod assignment_resolver;
mod color;
mod command_ext;
mod common;
mod compilation_error;
mod compilation_error_kind;
mod compilation_result_ext;
mod compiler;
mod config;
mod config_error;
mod count;
mod default;
mod dependency;
mod empty;
mod enclosure;
mod error;
mod error_result_ext;
mod expression;
mod fragment;
mod function;
mod function_context;
mod functions;
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
mod module;
mod name;
mod ordinal;
mod output;
mod output_error;
mod parameter;
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
mod search;
mod search_config;
mod search_error;
mod set;
mod setting;
mod settings;
mod shebang;
mod show_whitespace;
mod state;
mod string_literal;
mod subcommand;
mod table;
mod token;
mod token_kind;
mod use_color;
mod variables;
mod verbosity;
mod warning;

pub use crate::run::run;

#[cfg(feature = "summary")]
pub mod summary;
