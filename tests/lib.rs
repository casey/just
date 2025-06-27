use {
  crate::{
    assert_stdout::assert_stdout,
    assert_success::assert_success,
    tempdir::tempdir,
    test::{assert_eval_eq, Output, Test},
  },
  executable_path::executable_path,
  just::{unindent, Response},
  libc::{EXIT_FAILURE, EXIT_SUCCESS},
  pretty_assertions::Comparison,
  regex::Regex,
  serde::{Deserialize, Serialize},
  serde_json::{json, Value},
  std::{
    collections::BTreeMap,
    env::{self, consts::EXE_SUFFIX},
    error::Error,
    fmt::Debug,
    fs,
    io::Write,
    iter,
    path::{Path, PathBuf, MAIN_SEPARATOR, MAIN_SEPARATOR_STR},
    process::{Command, Stdio},
    str,
  },
  tempfile::TempDir,
  temptree::{temptree, tree, Tree},
  which::which,
};

#[cfg(not(windows))]
use std::{
  thread,
  time::{Duration, Instant},
};

fn default<T: Default>() -> T {
  Default::default()
}

#[macro_use]
mod test;

mod alias;
mod alias_style;
mod allow_duplicate_recipes;
mod allow_duplicate_variables;
mod allow_missing;
mod assert_stdout;
mod assert_success;
mod assertions;
mod assignment;
mod attributes;
mod backticks;
mod byte_order_mark;
mod changelog;
mod choose;
mod command;
mod completions;
mod conditional;
mod confirm;
mod constants;
mod datetime;
mod delimiters;
mod directories;
mod dotenv;
mod edit;
mod equals;
mod error_messages;
mod evaluate;
mod examples;
mod explain;
mod export;
mod fallback;
mod format;
mod functions;
#[cfg(unix)]
mod global;
mod groups;
mod ignore_comments;
mod imports;
mod init;
mod invocation_directory;
mod json;
mod line_prefixes;
mod list;
mod logical_operators;
mod man;
mod misc;
mod modules;
mod multibyte_char;
mod newline_escape;
mod no_aliases;
mod no_cd;
mod no_dependencies;
mod no_exit_message;
mod os_attributes;
mod parameters;
mod parser;
mod positional_arguments;
mod private;
mod quiet;
mod quote;
mod readme;
mod recipe_dependencies;
mod recursion_limit;
mod regexes;
mod request;
mod run;
mod script;
mod search;
mod search_arguments;
mod shadowing_parameters;
mod shebang;
mod shell;
mod shell_expansion;
mod show;
#[cfg(unix)]
mod signals;
mod slash_operator;
mod string;
mod subsequents;
mod summary;
mod tempdir;
mod timestamps;
mod undefined_variables;
mod unexport;
mod unstable;
mod which_function;
#[cfg(windows)]
mod windows;
#[cfg(target_family = "windows")]
mod windows_shell;
mod working_directory;

fn path(s: &str) -> String {
  if cfg!(windows) {
    s.replace('/', "\\")
  } else {
    s.into()
  }
}

fn path_for_regex(s: &str) -> String {
  if cfg!(windows) {
    s.replace('/', "\\\\")
  } else {
    s.into()
  }
}
