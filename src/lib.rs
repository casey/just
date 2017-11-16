#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate tempdir;
extern crate itertools;
extern crate ansi_term;
extern crate unicode_width;
extern crate edit_distance;
extern crate libc;
extern crate brev;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod unit;

#[cfg(test)]
mod integration;

#[cfg(test)]
mod search;

mod platform;
mod app;
mod color;
mod compilation_error;
mod runtime_error;
mod formatting;

use compilation_error::{CompilationError, CompilationErrorKind};
use runtime_error::RuntimeError;

mod prelude {
  pub use libc::{EXIT_FAILURE, EXIT_SUCCESS};
  pub use regex::Regex;
  pub use std::io::prelude::*;
  pub use std::path::{Path, PathBuf};
  pub use std::{cmp, env, fs, fmt, io, iter, process};

  pub fn default<T: Default>() -> T {
    Default::default()
  }
}

use prelude::*;

pub use app::app;

use brev::output;
use color::Color;
use platform::{Platform, PlatformInterface};
use std::borrow::Cow;
use std::collections::{BTreeMap as Map, BTreeSet as Set};
use std::fmt::Display;
use std::ops::Range;

const DEFAULT_SHELL: &'static str = "sh";

trait Slurp {
  fn slurp(&mut self) -> Result<String, std::io::Error>;
}

impl Slurp for fs::File {
  fn slurp(&mut self) -> Result<String, std::io::Error> {
    let mut destination = String::new();
    self.read_to_string(&mut destination)?;
    Ok(destination)
  }
}

/// Split a shebang line into a command and an optional argument
fn split_shebang(shebang: &str) -> Option<(&str, Option<&str>)> {
  lazy_static! {
    static ref EMPTY:    Regex = re(r"^#!\s*$");
    static ref SIMPLE:   Regex = re(r"^#!(\S+)\s*$");
    static ref ARGUMENT: Regex = re(r"^#!(\S+)\s+(\S.*?)?\s*$");
  }

  if EMPTY.is_match(shebang) {
    Some(("", None))
  } else if let Some(captures) = SIMPLE.captures(shebang) {
    Some((captures.get(1).unwrap().as_str(), None))
  } else if let Some(captures) = ARGUMENT.captures(shebang) {
    Some((captures.get(1).unwrap().as_str(), Some(captures.get(2).unwrap().as_str())))
  } else {
    None
  }
}

fn re(pattern: &str) -> Regex {
  Regex::new(pattern).unwrap()
}

fn empty<T, C: iter::FromIterator<T>>() -> C {
  iter::empty().collect()
}

fn contains<T: PartialOrd + Copy>(range: &Range<T>,  i: T) -> bool {
  i >= range.start && i < range.end
}

#[derive(PartialEq, Debug)]
struct Recipe<'a> {
  dependencies:      Vec<&'a str>,
  dependency_tokens: Vec<Token<'a>>,
  doc:               Option<&'a str>,
  line_number:       usize,
  lines:             Vec<Vec<Fragment<'a>>>,
  name:              &'a str,
  parameters:        Vec<Parameter<'a>>,
  private:           bool,
  quiet:             bool,
  shebang:           bool,
}

#[derive(PartialEq, Debug)]
struct Parameter<'a> {
  default:  Option<String>,
  name:     &'a str,
  token:    Token<'a>,
  variadic: bool,
}

impl<'a> Display for Parameter<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let color = Color::fmt(f);
    if self.variadic {
      write!(f, "{}", color.annotation().paint("+"))?;
    }
    write!(f, "{}", color.parameter().paint(self.name))?;
    if let Some(ref default) = self.default {
      let escaped = default.chars().flat_map(char::escape_default).collect::<String>();;
      write!(f, r#"='{}'"#, color.string().paint(&escaped))?;
    }
    Ok(())
  }
}

#[derive(PartialEq, Debug)]
enum Fragment<'a> {
  Text{text: Token<'a>},
  Expression{expression: Expression<'a>},
}

impl<'a> Fragment<'a> {
  fn continuation(&self) -> bool {
    match *self {
      Fragment::Text{ref text} => text.lexeme.ends_with('\\'),
      _ => false,
    }
  }
}

#[derive(PartialEq, Debug)]
enum Expression<'a> {
  Variable{name: &'a str, token: Token<'a>},
  String{cooked_string: CookedString<'a>},
  Backtick{raw: &'a str, token: Token<'a>},
  Concatination{lhs: Box<Expression<'a>>, rhs: Box<Expression<'a>>},
}

impl<'a> Expression<'a> {
  fn variables(&'a self) -> Variables<'a> {
    Variables {
      stack: vec![self],
    }
  }
}

struct Variables<'a> {
  stack: Vec<&'a Expression<'a>>,
}

impl<'a> Iterator for Variables<'a> {
  type Item = &'a Token<'a>;

  fn next(&mut self) -> Option<&'a Token<'a>> {
    match self.stack.pop() {
      None | Some(&Expression::String{..}) | Some(&Expression::Backtick{..}) => None,
      Some(&Expression::Variable{ref token,..})          => Some(token),
      Some(&Expression::Concatination{ref lhs, ref rhs}) => {
        self.stack.push(lhs);
        self.stack.push(rhs);
        self.next()
      }
    }
  }
}

impl<'a> Display for Expression<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Expression::Backtick     {raw, ..          } => write!(f, "`{}`", raw)?,
      Expression::Concatination{ref lhs, ref rhs } => write!(f, "{} + {}", lhs, rhs)?,
      Expression::String       {ref cooked_string} => write!(f, "\"{}\"", cooked_string.raw)?,
      Expression::Variable     {name, ..         } => write!(f, "{}", name)?,
    }
    Ok(())
  }
}

/// Return a `RuntimeError::Signal` if the process was terminated by a signal,
/// otherwise return an `RuntimeError::UnknownFailure`
fn error_from_signal(
  recipe:      &str,
  line_number: Option<usize>,
  exit_status: process::ExitStatus
) -> RuntimeError {
  match Platform::signal_from_exit_status(exit_status) {
    Some(signal) => RuntimeError::Signal{recipe: recipe, line_number: line_number, signal: signal},
    None => RuntimeError::UnknownFailure{recipe: recipe, line_number: line_number},
  }
}

fn export_env<'a>(
  command: &mut process::Command,
  scope:   &Map<&'a str, String>,
  exports: &Set<&'a str>,
) -> Result<(), RuntimeError<'a>> {
  for name in exports {
    if let Some(value) = scope.get(name) {
      command.env(name, value);
    } else {
      return Err(RuntimeError::InternalError {
        message: format!("scope does not contain exported variable `{}`",  name),
      });
    }
  }
  Ok(())
}

fn run_backtick<'a>(
  raw:     &str,
  token:   &Token<'a>,
  scope:   &Map<&'a str, String>,
  exports: &Set<&'a str>,
  quiet:   bool,
) -> Result<String, RuntimeError<'a>> {
  let mut cmd = process::Command::new(DEFAULT_SHELL);

  export_env(&mut cmd, scope, exports)?;

  cmd.arg("-cu")
     .arg(raw);

  cmd.stderr(if quiet {
    process::Stdio::null()
  } else {
    process::Stdio::inherit()
  });

  output(cmd).map_err(|output_error| RuntimeError::Backtick{token: token.clone(), output_error})
}

impl<'a> Recipe<'a> {
  fn argument_range(&self) -> Range<usize> {
    self.min_arguments()..self.max_arguments() + 1
  }

  fn min_arguments(&self) -> usize {
    self.parameters.iter().filter(|p| !p.default.is_some()).count()
  }

  fn max_arguments(&self) -> usize {
    if self.parameters.iter().any(|p| p.variadic) {
      std::usize::MAX - 1
    } else {
      self.parameters.len()
    }
  }

  fn run(
    &self,
    arguments: &[&'a str],
    scope:     &Map<&'a str, String>,
    exports:   &Set<&'a str>,
    options:   &RunOptions,
  ) -> Result<(), RuntimeError<'a>> {
    if options.verbose {
      let color = options.color.stderr().banner();
      eprintln!("{}===> Running recipe `{}`...{}", color.prefix(), self.name, color.suffix());
    }

    let mut argument_map = Map::new();

    let mut rest = arguments;
    for parameter in &self.parameters {
      let value = if rest.is_empty() {
        match parameter.default {
          Some(ref default) => Cow::Borrowed(default.as_str()),
          None => return Err(RuntimeError::InternalError{
            message: "missing parameter without default".to_string()
          }),
        }
      } else if parameter.variadic {
        let value = Cow::Owned(rest.to_vec().join(" "));
        rest = &[];
        value
      } else {
        let value = Cow::Borrowed(rest[0]);
        rest = &rest[1..];
        value
      };
      argument_map.insert(parameter.name, value);
    }

    let mut evaluator = Evaluator {
      evaluated:   empty(),
      scope:       scope,
      exports:     exports,
      assignments: &empty(),
      overrides:   &empty(),
      quiet:       options.quiet,
    };

    if self.shebang {
      let mut evaluated_lines = vec![];
      for line in &self.lines {
        evaluated_lines.push(evaluator.evaluate_line(line, &argument_map)?);
      }

      if options.dry_run || self.quiet {
        for line in &evaluated_lines {
          eprintln!("{}", line);
        }
      }

      if options.dry_run {
        return Ok(());
      }

      let tmp = tempdir::TempDir::new("just")
        .map_err(|error| RuntimeError::TmpdirIoError{recipe: self.name, io_error: error})?;
      let mut path = tmp.path().to_path_buf();
      path.push(self.name);
      {
        let mut f = fs::File::create(&path)
          .map_err(|error| RuntimeError::TmpdirIoError{recipe: self.name, io_error: error})?;
        let mut text = String::new();
        // add the shebang
        text += &evaluated_lines[0];
        text += "\n";
        // add blank lines so that lines in the generated script
        // have the same line number as the corresponding lines
        // in the justfile
        for _ in 1..(self.line_number + 2) {
          text += "\n"
        }
        for line in &evaluated_lines[1..] {
          text += line;
          text += "\n";
        }
        f.write_all(text.as_bytes())
         .map_err(|error| RuntimeError::TmpdirIoError{recipe: self.name, io_error: error})?;
      }

      // make the script executable
      Platform::set_execute_permission(&path)
        .map_err(|error| RuntimeError::TmpdirIoError{recipe: self.name, io_error: error})?;

      let shebang_line = evaluated_lines.first()
        .ok_or_else(|| RuntimeError::InternalError {
          message: "evaluated_lines was empty".to_string()
        })?;

      let (shebang_command, shebang_argument) = split_shebang(shebang_line)
        .ok_or_else(|| RuntimeError::InternalError {
          message: format!("bad shebang line: {}", shebang_line)
        })?;

      // create a command to run the script
      let mut command = Platform::make_shebang_command(&path, shebang_command, shebang_argument)
        .map_err(|output_error| RuntimeError::Cygpath{recipe: self.name, output_error: output_error})?;

      // export environment variables
      export_env(&mut command, scope, exports)?;

      // run it!
      match command.status() {
        Ok(exit_status) => if let Some(code) = exit_status.code() {
          if code != 0 {
            return Err(RuntimeError::Code{recipe: self.name, line_number: None, code: code})
          }
        } else {
          return Err(error_from_signal(self.name, None, exit_status))
        },
        Err(io_error) => return Err(RuntimeError::Shebang {
          recipe:   self.name,
          command:  shebang_command.to_string(),
          argument: shebang_argument.map(String::from),
          io_error: io_error
        })
      };
    } else {
      let mut lines = self.lines.iter().peekable();
      let mut line_number = self.line_number + 1;
      loop {
        if lines.peek().is_none() {
          break;
        }
        let mut evaluated = String::new();
        loop {
          if lines.peek().is_none() {
            break;
          }
          let line = lines.next().unwrap();
          line_number += 1;
          evaluated += &evaluator.evaluate_line(line, &argument_map)?;
          if line.last().map(Fragment::continuation).unwrap_or(false) {
            evaluated.pop();
          } else {
            break;
          }
        }
        let mut command = evaluated.as_str();
        let quiet_command = command.starts_with('@');
        if quiet_command {
          command = &command[1..];
        }

        if command == "" {
          continue;
        }

        if options.dry_run || options.verbose || !((quiet_command ^ self.quiet) || options.quiet) {
          let color = if options.highlight {
            options.color.command()
          } else {
            options.color
          };
          eprintln!("{}", color.stderr().paint(command));
        }

        if options.dry_run {
          continue;
        }

        let mut cmd = process::Command::new(options.shell.unwrap_or(DEFAULT_SHELL));

        cmd.arg("-cu").arg(command);

        if options.quiet {
          cmd.stderr(process::Stdio::null());
          cmd.stdout(process::Stdio::null());
        }

        export_env(&mut cmd, scope, exports)?;

        match cmd.status() {
          Ok(exit_status) => if let Some(code) = exit_status.code() {
            if code != 0 {
              return Err(RuntimeError::Code{
                recipe: self.name, line_number: Some(line_number), code: code
              });
            }
          } else {
            return Err(error_from_signal(self.name, Some(line_number), exit_status));
          },
          Err(io_error) => return Err(RuntimeError::IoError{
            recipe: self.name, io_error: io_error}),
        };
      }
    }
    Ok(())
  }
}

impl<'a> Display for Recipe<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    if let Some(doc) = self.doc {
      writeln!(f, "# {}", doc)?;
    }
    write!(f, "{}", self.name)?;
    for parameter in &self.parameters {
      write!(f, " {}", parameter)?;
    }
    write!(f, ":")?;
    for dependency in &self.dependencies {
      write!(f, " {}", dependency)?;
    }

    for (i, pieces) in self.lines.iter().enumerate() {
      if i == 0 {
        writeln!(f, "")?;
      }
      for (j, piece) in pieces.iter().enumerate() {
        if j == 0 {
          write!(f, "    ")?;
        }
        match *piece {
          Fragment::Text{ref text} => write!(f, "{}", text.lexeme)?,
          Fragment::Expression{ref expression, ..} =>
            write!(f, "{}{}{}", "{{", expression, "}}")?,
        }
      }
      if i + 1 < self.lines.len() {
        write!(f, "\n")?;
      }
    }
    Ok(())
  }
}

fn resolve_recipes<'a>(
  recipes:     &Map<&'a str, Recipe<'a>>,
  assignments: &Map<&'a str, Expression<'a>>,
  text:        &'a str,
) -> Result<(), CompilationError<'a>> {
  let mut resolver = Resolver {
    seen:              empty(),
    stack:             empty(),
    resolved:          empty(),
    recipes:           recipes,
  };

  for recipe in recipes.values() {
    resolver.resolve(recipe)?;
    resolver.seen = empty();
  }

  for recipe in recipes.values() {
    for line in &recipe.lines {
      for fragment in line {
        if let Fragment::Expression{ref expression, ..} = *fragment {
          for variable in expression.variables() {
            let name = variable.lexeme;
            let undefined = !assignments.contains_key(name)
              && !recipe.parameters.iter().any(|p| p.name == name);
            if undefined {
              // There's a borrow issue here that seems too difficult to solve.
              // The error derived from the variable token has too short a lifetime,
              // so we create a new error from its contents, which do live long
              // enough.
              //
              // I suspect the solution here is to give recipes, pieces, and expressions
              // two lifetime parameters instead of one, with one being the lifetime
              // of the struct, and the second being the lifetime of the tokens
              // that it contains
              let error = variable.error(CompilationErrorKind::UndefinedVariable{variable: name});
              return Err(CompilationError {
                text:   text,
                index:  error.index,
                line:   error.line,
                column: error.column,
                width:  error.width,
                kind:   CompilationErrorKind::UndefinedVariable {
                  variable: &text[error.index..error.index + error.width.unwrap()],
                }
              });
            }
          }
        }
      }
    }
  }

  Ok(())
}

struct Resolver<'a: 'b, 'b> {
  stack:    Vec<&'a str>,
  seen:     Set<&'a str>,
  resolved: Set<&'a str>,
  recipes:  &'b Map<&'a str, Recipe<'a>>,
}

impl<'a, 'b> Resolver<'a, 'b> {
  fn resolve(&mut self, recipe: &Recipe<'a>) -> Result<(), CompilationError<'a>> {
    if self.resolved.contains(recipe.name) {
      return Ok(())
    }
    self.stack.push(recipe.name);
    self.seen.insert(recipe.name);
    for dependency_token in &recipe.dependency_tokens {
      match self.recipes.get(dependency_token.lexeme) {
        Some(dependency) => if !self.resolved.contains(dependency.name) {
          if self.seen.contains(dependency.name) {
            let first = self.stack[0];
            self.stack.push(first);
            return Err(dependency_token.error(CompilationErrorKind::CircularRecipeDependency {
              recipe: recipe.name,
              circle: self.stack.iter()
                .skip_while(|name| **name != dependency.name)
                .cloned().collect()
            }));
          }
          self.resolve(dependency)?;
        },
        None => return Err(dependency_token.error(CompilationErrorKind::UnknownDependency {
          recipe:  recipe.name,
          unknown: dependency_token.lexeme
        })),
      }
    }
    self.resolved.insert(recipe.name);
    self.stack.pop();
    Ok(())
  }
}

fn resolve_assignments<'a>(
  assignments:       &Map<&'a str, Expression<'a>>,
  assignment_tokens: &Map<&'a str, Token<'a>>,
) -> Result<(), CompilationError<'a>> {

  let mut resolver = AssignmentResolver {
    assignments:       assignments,
    assignment_tokens: assignment_tokens,
    stack:             empty(),
    seen:              empty(),
    evaluated:         empty(),
  };

  for name in assignments.keys() {
    resolver.resolve_assignment(name)?;
  }

  Ok(())
}

struct AssignmentResolver<'a: 'b, 'b> {
  assignments:       &'b Map<&'a str, Expression<'a>>,
  assignment_tokens: &'b Map<&'a str, Token<'a>>,
  stack:             Vec<&'a str>,
  seen:              Set<&'a str>,
  evaluated:         Set<&'a str>,
}

impl<'a: 'b, 'b> AssignmentResolver<'a, 'b> {
  fn resolve_assignment(&mut self, name: &'a str) -> Result<(), CompilationError<'a>> {
    if self.evaluated.contains(name) {
      return Ok(());
    }

    self.seen.insert(name);
    self.stack.push(name);

    if let Some(expression) = self.assignments.get(name) {
      self.resolve_expression(expression)?;
      self.evaluated.insert(name);
    } else {
      return Err(internal_error(format!("attempted to resolve unknown assignment `{}`", name)));
    }
    Ok(())
  }

  fn resolve_expression(&mut self, expression: &Expression<'a>) -> Result<(), CompilationError<'a>> {
    match *expression {
      Expression::Variable{name, ref token} => {
        if self.evaluated.contains(name) {
          return Ok(());
        } else if self.seen.contains(name) {
          let token = &self.assignment_tokens[name];
          self.stack.push(name);
          return Err(token.error(CompilationErrorKind::CircularVariableDependency {
            variable: name,
            circle:   self.stack.clone(),
          }));
        } else if self.assignments.contains_key(name) {
          self.resolve_assignment(name)?;
        } else {
          return Err(token.error(CompilationErrorKind::UndefinedVariable{variable: name}));
        }
      }
      Expression::Concatination{ref lhs, ref rhs} => {
        self.resolve_expression(lhs)?;
        self.resolve_expression(rhs)?;
      }
      Expression::String{..} | Expression::Backtick{..} => {}
    }
    Ok(())
  }
}

fn evaluate_assignments<'a>(
  assignments: &Map<&'a str, Expression<'a>>,
  overrides:   &Map<&str, &str>,
  quiet:       bool,
) -> Result<Map<&'a str, String>, RuntimeError<'a>> {
  let mut evaluator = Evaluator {
    assignments: assignments,
    evaluated:   empty(),
    exports:     &empty(),
    overrides:   overrides,
    quiet:       quiet,
    scope:       &empty(),
  };

  for name in assignments.keys() {
    evaluator.evaluate_assignment(name)?;
  }

  Ok(evaluator.evaluated)
}

struct Evaluator<'a: 'b, 'b> {
  assignments: &'b Map<&'a str, Expression<'a>>,
  evaluated:   Map<&'a str, String>,
  exports:     &'b Set<&'a str>,
  overrides:   &'b Map<&'b str, &'b str>,
  quiet:       bool,
  scope:       &'b Map<&'a str, String>,
}

impl<'a, 'b> Evaluator<'a, 'b> {
  fn evaluate_line(
    &mut self,
    line:      &[Fragment<'a>],
    arguments: &Map<&str, Cow<str>>
  ) -> Result<String, RuntimeError<'a>> {
    let mut evaluated = String::new();
    for fragment in line {
      match *fragment {
        Fragment::Text{ref text} => evaluated += text.lexeme,
        Fragment::Expression{ref expression} => {
          evaluated += &self.evaluate_expression(expression, arguments)?;
        }
      }
    }
    Ok(evaluated)
  }

  fn evaluate_assignment(&mut self, name: &'a str) -> Result<(), RuntimeError<'a>> {
    if self.evaluated.contains_key(name) {
      return Ok(());
    }

    if let Some(expression) = self.assignments.get(name) {
      if let Some(value) = self.overrides.get(name) {
        self.evaluated.insert(name, value.to_string());
      } else {
        let value = self.evaluate_expression(expression, &empty())?;
        self.evaluated.insert(name, value);
      }
    } else {
      return Err(RuntimeError::InternalError {
        message: format!("attempted to evaluated unknown assignment {}", name)
      });
    }

    Ok(())
  }

  fn evaluate_expression(
    &mut self,
    expression: &Expression<'a>,
    arguments: &Map<&str, Cow<str>>
  ) -> Result<String, RuntimeError<'a>> {
    Ok(match *expression {
      Expression::Variable{name, ..} => {
        if self.evaluated.contains_key(name) {
          self.evaluated[name].clone()
        } else if self.scope.contains_key(name) {
          self.scope[name].clone()
        } else if self.assignments.contains_key(name) {
          self.evaluate_assignment(name)?;
          self.evaluated[name].clone()
        } else if arguments.contains_key(name) {
          arguments[name].to_string()
        } else {
          return Err(RuntimeError::InternalError {
            message: format!("attempted to evaluate undefined variable `{}`", name)
          });
        }
      }
      Expression::String{ref cooked_string} => cooked_string.cooked.clone(),
      Expression::Backtick{raw, ref token} => {
        run_backtick(raw, token, self.scope, self.exports, self.quiet)?
      }
      Expression::Concatination{ref lhs, ref rhs} => {
        self.evaluate_expression(lhs, arguments)?
          +
        &self.evaluate_expression(rhs, arguments)?
      }
    })
  }
}

fn internal_error(message: String) -> CompilationError<'static> {
  CompilationError {
    text:   "",
    index:  0,
    line:   0,
    column: 0,
    width:  None,
    kind:   CompilationErrorKind::InternalError { message: message }
  }
}

fn mixed_whitespace(text: &str) -> bool {
  !(text.chars().all(|c| c == ' ') || text.chars().all(|c| c == '\t'))
}

#[derive(PartialEq, Debug)]
struct CookedString<'a> {
  raw:    &'a str,
  cooked: String,
}

fn cook_string<'a>(token: &Token<'a>) -> Result<CookedString<'a>, CompilationError<'a>> {
  let raw = &token.lexeme[1..token.lexeme.len()-1];

  if let RawString = token.kind {
    Ok(CookedString{raw: raw, cooked: raw.to_string()})
  } else if let StringToken = token.kind {
    let mut cooked = String::new();
    let mut escape = false;
    for c in raw.chars() {
      if escape {
        match c {
          'n'   => cooked.push('\n'),
          'r'   => cooked.push('\r'),
          't'   => cooked.push('\t'),
          '\\'  => cooked.push('\\'),
          '"'   => cooked.push('"'),
          other => return Err(token.error(CompilationErrorKind::InvalidEscapeSequence {
            character: other,
          })),
        }
        escape = false;
        continue;
      }
      if c == '\\' {
        escape = true;
        continue;
      }
      cooked.push(c);
    }
    Ok(CookedString{raw: raw, cooked: cooked})
  } else {
    Err(token.error(CompilationErrorKind::InternalError{
      message: "cook_string() called on non-string token".to_string()
    }))
  }
}

struct Justfile<'a> {
  recipes:     Map<&'a str, Recipe<'a>>,
  assignments: Map<&'a str, Expression<'a>>,
  exports:     Set<&'a str>,
}

#[derive(Default)]
struct RunOptions<'a> {
  dry_run:   bool,
  evaluate:  bool,
  highlight: bool,
  overrides: Map<&'a str, &'a str>,
  quiet:     bool,
  shell:     Option<&'a str>,
  color:     Color,
  verbose:   bool,
}

impl<'a, 'b> Justfile<'a> where 'a: 'b {
  fn first(&self) -> Option<&Recipe> {
    let mut first: Option<&Recipe> = None;
    for recipe in self.recipes.values() {
      if let Some(first_recipe) = first {
        if recipe.line_number < first_recipe.line_number {
          first = Some(recipe)
        }
      } else {
        first = Some(recipe);
      }
    }
    first
  }

  fn count(&self) -> usize {
    self.recipes.len()
  }

  fn suggest(&self, name: &str) -> Option<&'a str> {
    let mut suggestions = self.recipes.keys()
      .map(|suggestion| (edit_distance::edit_distance(suggestion, name), suggestion))
      .collect::<Vec<_>>();
    suggestions.sort();
    if let Some(&(distance, suggestion)) = suggestions.first() {
      if distance < 3 {
        return Some(suggestion)
      }
    }
    None
  }

  fn run(
    &'a self,
    arguments: &[&'a str],
    options:   &RunOptions<'a>,
  ) -> Result<(), RuntimeError<'a>> {
    let unknown_overrides = options.overrides.keys().cloned()
      .filter(|name| !self.assignments.contains_key(name))
      .collect::<Vec<_>>();

    if !unknown_overrides.is_empty() {
      return Err(RuntimeError::UnknownOverrides{overrides: unknown_overrides});
    }

    let scope = evaluate_assignments(&self.assignments, &options.overrides, options.quiet)?;
    if options.evaluate {
      let mut width = 0;
      for name in scope.keys() {
        width = cmp::max(name.len(), width);
      }

      for (name, value) in scope {
        println!("{0:1$} = \"{2}\"", name, width, value);
      }
      return Ok(());
    }

    let mut missing = vec![];
    let mut grouped = vec![];
    let mut rest    = arguments;

    while let Some((argument, mut tail)) = rest.split_first() {
      if let Some(recipe) = self.recipes.get(argument) {
        if recipe.parameters.is_empty() {
          grouped.push((recipe, &tail[0..0]));
        } else {
          let argument_range = recipe.argument_range();
          let argument_count = cmp::min(tail.len(), recipe.max_arguments());
          if !contains(&argument_range, argument_count) {
            return Err(RuntimeError::ArgumentCountMismatch {
              recipe: recipe.name,
              found:  tail.len(),
              min:    recipe.min_arguments(),
              max:    recipe.max_arguments(),
            });
          }
          grouped.push((recipe, &tail[0..argument_count]));
          tail = &tail[argument_count..];
        }
      } else {
        missing.push(*argument);
      }
      rest = tail;
    }

    if !missing.is_empty() {
      let suggestion = if missing.len() == 1 {
        self.suggest(missing.first().unwrap())
      } else {
        None
      };
      return Err(RuntimeError::UnknownRecipes{recipes: missing, suggestion: suggestion});
    }

    let mut ran = empty();
    for (recipe, arguments) in grouped {
      self.run_recipe(recipe, arguments, &scope, &mut ran, options)?
    }

    Ok(())
  }

  fn run_recipe<'c>(
    &'c self,
    recipe:    &Recipe<'a>,
    arguments: &[&'a str],
    scope:     &Map<&'c str, String>,
    ran:       &mut Set<&'a str>,
    options:   &RunOptions<'a>,
  ) -> Result<(), RuntimeError> {
    for dependency_name in &recipe.dependencies {
      if !ran.contains(dependency_name) {
        self.run_recipe(&self.recipes[dependency_name], &[], scope, ran, options)?;
      }
    }
    recipe.run(arguments, scope, &self.exports, options)?;
    ran.insert(recipe.name);
    Ok(())
  }
}

impl<'a> Display for Justfile<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let mut items = self.recipes.len() + self.assignments.len();
    for (name, expression) in &self.assignments {
      if self.exports.contains(name) {
        write!(f, "export ")?;
      }
      write!(f, "{} = {}", name, expression)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    for recipe in self.recipes.values() {
      write!(f, "{}", recipe)?;
      items -= 1;
      if items != 0 {
        write!(f, "\n\n")?;
      }
    }
    Ok(())
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
  index:  usize,
  line:   usize,
  column: usize,
  text:   &'a str,
  prefix: &'a str,
  lexeme: &'a str,
  kind:   TokenKind,
}

impl<'a> Token<'a> {
  fn error(&self, kind: CompilationErrorKind<'a>) -> CompilationError<'a> {
    CompilationError {
      text:   self.text,
      index:  self.index + self.prefix.len(),
      line:   self.line,
      column: self.column + self.prefix.len(),
      width:  Some(self.lexeme.len()),
      kind:   kind,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
  At,
  Backtick,
  Colon,
  Comment,
  Dedent,
  Eof,
  Eol,
  Equals,
  Indent,
  InterpolationEnd,
  InterpolationStart,
  Line,
  Name,
  Plus,
  RawString,
  StringToken,
  Text,
}

impl Display for TokenKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(f, "{}", match *self {
      Backtick           => "backtick",
      Colon              => "':'",
      Comment            => "comment",
      Dedent             => "dedent",
      Eof                => "end of file",
      Eol                => "end of line",
      Equals             => "'='",
      Indent             => "indent",
      InterpolationEnd   => "'}}'",
      InterpolationStart => "'{{'",
      Line               => "command",
      Name               => "name",
      Plus               => "'+'",
      At                 => "'@'",
      StringToken        => "string",
      RawString          => "raw string",
      Text               => "command text",
    })
  }
}

use TokenKind::*;

fn token(pattern: &str) -> Regex {
  let mut s = String::new();
  s += r"^(?m)([ \t]*)(";
  s += pattern;
  s += ")";
  re(&s)
}

fn tokenize(text: &str) -> Result<Vec<Token>, CompilationError> {
  lazy_static! {
    static ref BACKTICK:                  Regex = token(r"`[^`\n\r]*`"               );
    static ref COLON:                     Regex = token(r":"                         );
    static ref AT:                        Regex = token(r"@"                         );
    static ref COMMENT:                   Regex = token(r"#([^!\n\r].*)?$"           );
    static ref EOF:                       Regex = token(r"(?-m)$"                    );
    static ref EOL:                       Regex = token(r"\n|\r\n"                   );
    static ref EQUALS:                    Regex = token(r"="                         );
    static ref INTERPOLATION_END:         Regex = token(r"[}][}]"                    );
    static ref INTERPOLATION_START_TOKEN: Regex = token(r"[{][{]"                    );
    static ref NAME:                      Regex = token(r"([a-zA-Z_][a-zA-Z0-9_-]*)" );
    static ref PLUS:                      Regex = token(r"[+]"                       );
    static ref STRING:                    Regex = token("\""                         );
    static ref RAW_STRING:                Regex = token(r#"'[^']*'"#                 );
    static ref UNTERMINATED_RAW_STRING:   Regex = token(r#"'[^']*"#                  );
    static ref INDENT:                    Regex = re(r"^([ \t]*)[^ \t\n\r]"     );
    static ref INTERPOLATION_START:       Regex = re(r"^[{][{]"                 );
    static ref LEADING_TEXT:              Regex = re(r"^(?m)(.+?)[{][{]"        );
    static ref LINE:                      Regex = re(r"^(?m)[ \t]+[^ \t\n\r].*$");
    static ref TEXT:                      Regex = re(r"^(?m)(.+)"               );
  }

  #[derive(PartialEq)]
  enum State<'a> {
    Start,
    Indent(&'a str),
    Text,
    Interpolation,
  }

  fn indentation(text: &str) -> Option<&str> {
    INDENT.captures(text).map(|captures| captures.get(1).unwrap().as_str())
  }

  let mut tokens = vec![];
  let mut rest   = text;
  let mut index  = 0;
  let mut line   = 0;
  let mut column = 0;
  let mut state  = vec![State::Start];

  macro_rules! error {
    ($kind:expr) => {{
      Err(CompilationError {
        text:   text,
        index:  index,
        line:   line,
        column: column,
        width:  None,
        kind:   $kind,
      })
    }};
  }

  loop {
    if column == 0 {
      if let Some(kind) = match (state.last().unwrap(), indentation(rest)) {
        // ignore: was no indentation and there still isn't
        //         or current line is blank
        (&State::Start, Some("")) | (_, None) => {
          None
        }
        // indent: was no indentation, now there is
        (&State::Start, Some(current)) => {
          if mixed_whitespace(current) {
            return error!(CompilationErrorKind::MixedLeadingWhitespace{whitespace: current})
          }
          //indent = Some(current);
          state.push(State::Indent(current));
          Some(Indent)
        }
        // dedent: there was indentation and now there isn't
        (&State::Indent(_), Some("")) => {
          // indent = None;
          state.pop();
          Some(Dedent)
        }
        // was indentation and still is, check if the new indentation matches
        (&State::Indent(previous), Some(current)) => {
          if !current.starts_with(previous) {
            return error!(CompilationErrorKind::InconsistentLeadingWhitespace{
              expected: previous,
              found: current
            });
          }
          None
        }
        // at column 0 in some other state: this should never happen
        (&State::Text, _) | (&State::Interpolation, _) => {
          return error!(CompilationErrorKind::InternalError{
            message: "unexpected state at column 0".to_string()
          });
        }
      } {
        tokens.push(Token {
          index:  index,
          line:   line,
          column: column,
          text:   text,
          prefix: "",
          lexeme: "",
          kind:   kind,
        });
      }
    }

    // insert a dedent if we're indented and we hit the end of the file
    if &State::Start != state.last().unwrap() && EOF.is_match(rest) {
      tokens.push(Token {
        index:  index,
        line:   line,
        column: column,
        text:   text,
        prefix: "",
        lexeme: "",
        kind:   Dedent,
      });
    }

    let (prefix, lexeme, kind) =
    if let (0, &State::Indent(indent), Some(captures)) =
      (column, state.last().unwrap(), LINE.captures(rest)) {
      let line = captures.get(0).unwrap().as_str();
      if !line.starts_with(indent) {
        return error!(CompilationErrorKind::InternalError{message: "unexpected indent".to_string()});
      }
      state.push(State::Text);
      (&line[0..indent.len()], "", Line)
    } else if let Some(captures) = EOF.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Eof)
    } else if let State::Text = *state.last().unwrap() {
      if let Some(captures) = INTERPOLATION_START.captures(rest) {
        state.push(State::Interpolation);
        ("", captures.get(0).unwrap().as_str(), InterpolationStart)
      } else if let Some(captures) = LEADING_TEXT.captures(rest) {
        ("", captures.get(1).unwrap().as_str(), Text)
      } else if let Some(captures) = TEXT.captures(rest) {
        ("", captures.get(1).unwrap().as_str(), Text)
      } else if let Some(captures) = EOL.captures(rest) {
        state.pop();
        (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Eol)
      } else {
        return error!(CompilationErrorKind::InternalError{
          message: format!("Could not match token in text state: \"{}\"", rest)
        });
      }
    } else if let Some(captures) = INTERPOLATION_START_TOKEN.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), InterpolationStart)
    } else if let Some(captures) = INTERPOLATION_END.captures(rest) {
      if state.last().unwrap() == &State::Interpolation {
        state.pop();
      }
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), InterpolationEnd)
    } else if let Some(captures) = NAME.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Name)
    } else if let Some(captures) = EOL.captures(rest) {
      if state.last().unwrap() == &State::Interpolation {
        return error!(CompilationErrorKind::InternalError {
          message: "hit EOL while still in interpolation state".to_string()
        });
      }
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Eol)
    } else if let Some(captures) = BACKTICK.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Backtick)
    } else if let Some(captures) = COLON.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Colon)
    } else if let Some(captures) = AT.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), At)
    } else if let Some(captures) = PLUS.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Plus)
    } else if let Some(captures) = EQUALS.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Equals)
    } else if let Some(captures) = COMMENT.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Comment)
    } else if let Some(captures) = RAW_STRING.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), RawString)
    } else if UNTERMINATED_RAW_STRING.is_match(rest) {
      return error!(CompilationErrorKind::UnterminatedString);
    } else if let Some(captures) = STRING.captures(rest) {
      let prefix = captures.get(1).unwrap().as_str();
      let contents = &rest[prefix.len()+1..];
      if contents.is_empty() {
        return error!(CompilationErrorKind::UnterminatedString);
      }
      let mut len = 0;
      let mut escape = false;
      for c in contents.chars() {
        if c == '\n' || c == '\r' {
          return error!(CompilationErrorKind::UnterminatedString);
        } else if !escape && c == '"' {
          break;
        } else if !escape && c == '\\' {
          escape = true;
        } else if escape {
          escape = false;
        }
        len += c.len_utf8();
      }
      let start = prefix.len();
      let content_end = start + len + 1;
      if escape || content_end >= rest.len() {
        return error!(CompilationErrorKind::UnterminatedString);
      }
      (prefix, &rest[start..content_end + 1], StringToken)
    } else if rest.starts_with("#!") {
      return error!(CompilationErrorKind::OuterShebang)
    } else {
      return error!(CompilationErrorKind::UnknownStartOfToken)
    };

    tokens.push(Token {
      index:  index,
      line:   line,
      column: column,
      prefix: prefix,
      text:   text,
      lexeme: lexeme,
      kind:   kind,
    });

    let len = prefix.len() + lexeme.len();

    if len == 0 {
      let last = tokens.last().unwrap();
      match last.kind {
        Eof => {},
        _ => return Err(last.error(CompilationErrorKind::InternalError{
          message: format!("zero length token: {:?}", last)
        })),
      }
    }

    match tokens.last().unwrap().kind {
      Eol => {
        line += 1;
        column = 0;
      }
      Eof => {
        break;
      }
      RawString => {
        let lexeme_lines = lexeme.lines().count();
        line += lexeme_lines - 1;
        if lexeme_lines == 1 {
          column += len;
        } else {
          column = lexeme.lines().last().unwrap().len();
        }
      }
      _ => {
        column += len;
      }
    }

    rest = &rest[len..];
    index += len;
  }

  Ok(tokens)
}

fn compile(text: &str) -> Result<Justfile, CompilationError> {
  let tokens = tokenize(text)?;
  let parser = Parser {
    text:              text,
    tokens:            itertools::put_back(tokens),
    recipes:           empty(),
    assignments:       empty(),
    assignment_tokens: empty(),
    exports:           empty(),
  };
  parser.justfile()
}

struct Parser<'a> {
  text:              &'a str,
  tokens:            itertools::PutBack<std::vec::IntoIter<Token<'a>>>,
  recipes:           Map<&'a str, Recipe<'a>>,
  assignments:       Map<&'a str, Expression<'a>>,
  assignment_tokens: Map<&'a str, Token<'a>>,
  exports:           Set<&'a str>,
}

impl<'a> Parser<'a> {
  fn peek(&mut self, kind: TokenKind) -> bool {
    let next = self.tokens.next().unwrap();
    let result = next.kind == kind;
    self.tokens.put_back(next);
    result
  }

  fn accept(&mut self, kind: TokenKind) -> Option<Token<'a>> {
    if self.peek(kind) {
      self.tokens.next()
    } else {
      None
    }
  }

  fn accept_any(&mut self, kinds: &[TokenKind]) -> Option<Token<'a>> {
    for kind in kinds {
      if self.peek(*kind) {
        return self.tokens.next();
      }
    }
    None
  }

  fn accepted(&mut self, kind: TokenKind) -> bool {
    self.accept(kind).is_some()
  }

  fn expect(&mut self, kind: TokenKind) -> Option<Token<'a>> {
    if self.peek(kind) {
      self.tokens.next();
      None
    } else {
      self.tokens.next()
    }
  }

  fn expect_eol(&mut self) -> Option<Token<'a>> {
    self.accepted(Comment);
    if self.peek(Eol) {
      self.accept(Eol);
      None
    } else if self.peek(Eof) {
      None
    } else {
      self.tokens.next()
    }
  }

  fn unexpected_token(&self, found: &Token<'a>, expected: &[TokenKind]) -> CompilationError<'a> {
    found.error(CompilationErrorKind::UnexpectedToken {
      expected: expected.to_vec(),
      found:    found.kind,
    })
  }

  fn recipe(
    &mut self,
    name:  Token<'a>,
    doc:   Option<Token<'a>>,
    quiet: bool,
  ) -> Result<(), CompilationError<'a>> {
    if let Some(recipe) = self.recipes.get(name.lexeme) {
      return Err(name.error(CompilationErrorKind::DuplicateRecipe {
        recipe: recipe.name,
        first:  recipe.line_number
      }));
    }

    let mut parsed_parameter_with_default = false;
    let mut parsed_variadic_parameter = false;
    let mut parameters: Vec<Parameter> = vec![];
    loop {
      let plus = self.accept(Plus);

      let parameter = match self.accept(Name) {
        Some(parameter) => parameter,
        None            => if let Some(plus) = plus {
          return Err(self.unexpected_token(&plus, &[Name]));
        } else {
          break
        },
      };

      let variadic = plus.is_some();

      if parsed_variadic_parameter {
        return Err(parameter.error(CompilationErrorKind::ParameterFollowsVariadicParameter {
          parameter: parameter.lexeme,
        }));
      }

      if parameters.iter().any(|p| p.name == parameter.lexeme) {
        return Err(parameter.error(CompilationErrorKind::DuplicateParameter {
          recipe: name.lexeme, parameter: parameter.lexeme
        }));
      }

      let default;
      if self.accepted(Equals) {
        if let Some(string) = self.accept_any(&[StringToken, RawString]) {
          default = Some(cook_string(&string)?.cooked);
        } else {
          let unexpected = self.tokens.next().unwrap();
          return Err(self.unexpected_token(&unexpected, &[StringToken, RawString]));
        }
      } else {
        default = None
      }

      if parsed_parameter_with_default && default.is_none() {
        return Err(parameter.error(CompilationErrorKind::RequiredParameterFollowsDefaultParameter{
          parameter: parameter.lexeme,
        }));
      }

      parsed_parameter_with_default |= default.is_some();
      parsed_variadic_parameter = variadic;

      parameters.push(Parameter {
        default:  default,
        name:     parameter.lexeme,
        token:    parameter,
        variadic: variadic,
      });
    }

    if let Some(token) = self.expect(Colon) {
      // if we haven't accepted any parameters, an equals
      // would have been fine as part of an assignment
      if parameters.is_empty() {
        return Err(self.unexpected_token(&token, &[Name, Plus, Colon, Equals]));
      } else {
        return Err(self.unexpected_token(&token, &[Name, Plus, Colon]));
      }
    }

    let mut dependencies = vec![];
    let mut dependency_tokens = vec![];
    while let Some(dependency) = self.accept(Name) {
      if dependencies.contains(&dependency.lexeme) {
        return Err(dependency.error(CompilationErrorKind::DuplicateDependency {
          recipe:     name.lexeme,
          dependency: dependency.lexeme
        }));
      }
      dependencies.push(dependency.lexeme);
      dependency_tokens.push(dependency);
    }

    if let Some(token) = self.expect_eol() {
      return Err(self.unexpected_token(&token, &[Name, Eol, Eof]));
    }

    let mut lines: Vec<Vec<Fragment>> = vec![];
    let mut shebang = false;

    if self.accepted(Indent) {
      while !self.accepted(Dedent) {
        if self.accepted(Eol) {
          lines.push(vec![]);
          continue;
        }
        if let Some(token) = self.expect(Line) {
          return Err(token.error(CompilationErrorKind::InternalError{
            message: format!("Expected a line but got {}", token.kind)
          }))
        }
        let mut fragments = vec![];

        while !(self.accepted(Eol) || self.peek(Dedent)) {
          if let Some(token) = self.accept(Text) {
            if fragments.is_empty() {
              if lines.is_empty() {
                if token.lexeme.starts_with("#!") {
                  shebang = true;
                }
              } else if !shebang
                && !lines.last().and_then(|line| line.last())
                  .map(Fragment::continuation).unwrap_or(false)
                && (token.lexeme.starts_with(' ') || token.lexeme.starts_with('\t')) {
                return Err(token.error(CompilationErrorKind::ExtraLeadingWhitespace));
              }
            }
            fragments.push(Fragment::Text{text: token});
          } else if let Some(token) = self.expect(InterpolationStart) {
            return Err(self.unexpected_token(&token, &[Text, InterpolationStart, Eol]));
          } else {
            fragments.push(Fragment::Expression{
              expression: self.expression(true)?
            });
            if let Some(token) = self.expect(InterpolationEnd) {
              return Err(self.unexpected_token(&token, &[InterpolationEnd]));
            }
          }
        }

        lines.push(fragments);
      }
    }

    self.recipes.insert(name.lexeme, Recipe {
      line_number:       name.line,
      name:              name.lexeme,
      doc:               doc.map(|t| t.lexeme[1..].trim()),
      dependencies:      dependencies,
      dependency_tokens: dependency_tokens,
      parameters:        parameters,
      private:           &name.lexeme[0..1] == "_",
      lines:             lines,
      shebang:           shebang,
      quiet:             quiet,
    });

    Ok(())
  }

  fn expression(&mut self, interpolation: bool) -> Result<Expression<'a>, CompilationError<'a>> {
    let first = self.tokens.next().unwrap();
    let lhs = match first.kind {
      Name        => Expression::Variable {name: first.lexeme, token: first},
      Backtick    => Expression::Backtick {
        raw:   &first.lexeme[1..first.lexeme.len()-1],
        token: first
      },
      RawString | StringToken => {
        Expression::String{cooked_string: cook_string(&first)?}
      }
      _ => return Err(self.unexpected_token(&first, &[Name, StringToken])),
    };

    if self.accepted(Plus) {
      let rhs = self.expression(interpolation)?;
      Ok(Expression::Concatination{lhs: Box::new(lhs), rhs: Box::new(rhs)})
    } else if interpolation && self.peek(InterpolationEnd) {
      Ok(lhs)
    } else if let Some(token) = self.expect_eol() {
      if interpolation {
        return Err(self.unexpected_token(&token, &[Plus, Eol, InterpolationEnd]))
      } else {
        Err(self.unexpected_token(&token, &[Plus, Eol]))
      }
    } else {
      Ok(lhs)
    }
  }

  fn assignment(&mut self, name: Token<'a>, export: bool) -> Result<(), CompilationError<'a>> {
    if self.assignments.contains_key(name.lexeme) {
      return Err(name.error(CompilationErrorKind::DuplicateVariable {variable: name.lexeme}));
    }
    if export {
      self.exports.insert(name.lexeme);
    }
    let expression = self.expression(false)?;
    self.assignments.insert(name.lexeme, expression);
    self.assignment_tokens.insert(name.lexeme, name);
    Ok(())
  }

  fn justfile(mut self) -> Result<Justfile<'a>, CompilationError<'a>> {
    let mut doc = None;
    loop {
      match self.tokens.next() {
        Some(token) => match token.kind {
          Eof => break,
          Eol => {
            doc = None;
            continue;
          }
          Comment => {
            if let Some(token) = self.expect_eol() {
              return Err(token.error(CompilationErrorKind::InternalError {
                message: format!("found comment followed by {}", token.kind),
              }));
            }
            doc = Some(token);
          }
          At => if let Some(name) = self.accept(Name) {
            self.recipe(name, doc, true)?;
            doc = None;
          } else {
            let unexpected = &self.tokens.next().unwrap();
            return Err(self.unexpected_token(unexpected, &[Name]));
          },
          Name => if token.lexeme == "export" {
            let next = self.tokens.next().unwrap();
            if next.kind == Name && self.accepted(Equals) {
              self.assignment(next, true)?;
              doc = None;
            } else {
              self.tokens.put_back(next);
              self.recipe(token, doc, false)?;
              doc = None;
            }
          } else if self.accepted(Equals) {
            self.assignment(token, false)?;
            doc = None;
          } else {
            self.recipe(token, doc, false)?;
            doc = None;
          },
          _ => return Err(self.unexpected_token(&token, &[Name, At])),
        },
        None => return Err(CompilationError {
          text:   self.text,
          index:  0,
          line:   0,
          column: 0,
          width:  None,
          kind:   CompilationErrorKind::InternalError {
            message: "unexpected end of token stream".to_string()
          }
        }),
      }
    }

    if let Some(token) = self.tokens.next() {
      return Err(token.error(CompilationErrorKind::InternalError{
        message: format!("unexpected token remaining after parsing completed: {:?}", token.kind)
      }))
    }

    resolve_recipes(&self.recipes, &self.assignments, self.text)?;

    for recipe in self.recipes.values() {
      for parameter in &recipe.parameters {
        if self.assignments.contains_key(parameter.token.lexeme) {
          return Err(parameter.token.error(CompilationErrorKind::ParameterShadowsVariable {
            parameter: parameter.token.lexeme
          }));
        }
      }

      for dependency in &recipe.dependency_tokens {
        if !self.recipes[dependency.lexeme].parameters.is_empty() {
          return Err(dependency.error(CompilationErrorKind::DependencyHasParameters {
            recipe: recipe.name,
            dependency: dependency.lexeme,
          }));
        }
      }
    }

    resolve_assignments(&self.assignments, &self.assignment_tokens)?;

    Ok(Justfile {
      recipes:     self.recipes,
      assignments: self.assignments,
      exports:     self.exports,
    })
  }
}
