#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

extern crate ansi_term;
extern crate brev;
extern crate clap;
extern crate ctrlc;
extern crate dotenv;
extern crate edit_distance;
extern crate env_logger;
extern crate itertools;
extern crate libc;
extern crate regex;
extern crate target;
extern crate tempdir;
extern crate unicode_width;

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
mod configuration;
mod cooked_string;
mod expression;
mod fragment;
mod function;
mod interrupt_handler;
mod justfile;
mod lexer;
mod load_dotenv;
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
mod verbosity;

use common::*;

pub use run::run;
