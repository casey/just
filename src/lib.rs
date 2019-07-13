#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
mod testing;

#[cfg(fuzzing)]
pub mod fuzzing;

#[macro_use]
mod die;

mod alias;
mod alias_resolver;
mod assignment_evaluator;
mod assignment_resolver;
mod color;
mod command_ext;
mod common;
mod compilation_error;
mod compilation_error_kind;
mod configuration;
mod expression;
mod fragment;
mod function;
mod function_context;
mod functions;
mod interrupt_guard;
mod interrupt_handler;
mod justfile;
mod lexer;
mod load_dotenv;
mod misc;
mod ordinal;
mod output;
mod output_error;
mod parameter;
mod parser;
mod platform;
mod platform_interface;
mod position;
mod range_ext;
mod recipe;
mod recipe_context;
mod recipe_resolver;
mod run;
mod runtime_error;
mod search;
mod search_error;
mod shebang;
mod state;
mod string_literal;
mod token;
mod token_kind;
mod use_color;
mod variables;
mod verbosity;

pub use crate::run::run;

pub mod summary;
