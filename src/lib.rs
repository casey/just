#[cfg(test)]
mod unit;

#[cfg(test)]
mod integration;

mod app;

pub use app::app;

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate tempdir;
extern crate itertools;
extern crate ansi_term;

use std::io::prelude::*;

use std::{fs, fmt, process, io};
use std::fmt::Display;
use regex::Regex;
use std::collections::{BTreeMap as Map, BTreeSet as Set};

use std::os::unix::fs::PermissionsExt;

macro_rules! warn {
  ($($arg:tt)*) => {{
    extern crate std;
    use std::io::prelude::*;
    let _ = writeln!(&mut std::io::stderr(), $($arg)*);
  }};
}

macro_rules! die {
  ($($arg:tt)*) => {{
    extern crate std;
    warn!($($arg)*);
    std::process::exit(-1)
  }};
}

trait Slurp {
  fn slurp(&mut self) -> Result<String, std::io::Error>;
}

impl Slurp for fs::File {
  fn slurp(&mut self) -> Result<String, std::io::Error> {
    let mut destination = String::new();
    try!(self.read_to_string(&mut destination));
    Ok(destination)
  }
}

fn re(pattern: &str) -> Regex {
  Regex::new(pattern).unwrap()
}

#[derive(PartialEq, Debug)]
struct Recipe<'a> {
  line_number:       usize,
  name:              &'a str,
  lines:             Vec<Vec<Fragment<'a>>>,
  dependencies:      Vec<&'a str>,
  dependency_tokens: Vec<Token<'a>>,
  parameters:        Vec<&'a str>,
  parameter_tokens:  Vec<Token<'a>>,
  shebang:           bool,
}

#[derive(PartialEq, Debug)]
enum Fragment<'a> {
  Text{text: Token<'a>},
  Expression{expression: Expression<'a>},
}

#[derive(PartialEq, Debug)]
enum Expression<'a> {
  Variable{name: &'a str, token: Token<'a>},
  String{raw: &'a str, cooked: String},
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
      Expression::Backtick     {raw, ..         } => try!(write!(f, "`{}`", raw)),
      Expression::Concatination{ref lhs, ref rhs} => try!(write!(f, "{} + {}", lhs, rhs)),
      Expression::String       {raw, ..         } => try!(write!(f, "\"{}\"", raw)),
      Expression::Variable     {name, ..        } => try!(write!(f, "{}", name)),
    }
    Ok(())
  }
}

#[cfg(unix)]
fn error_from_signal(recipe: &str, exit_status: process::ExitStatus) -> RunError {
  use std::os::unix::process::ExitStatusExt;
  match exit_status.signal() {
    Some(signal) => RunError::Signal{recipe: recipe, signal: signal},
    None => RunError::UnknownFailure{recipe: recipe},
  }
}

#[cfg(windows)]
fn error_from_signal(recipe: &str, exit_status: process::ExitStatus) -> RunError {
  RunError::UnknownFailure{recipe: recipe}
}

#[cfg(unix)]
fn backtick_error_from_signal<'a>(
  token:       &Token<'a>,
  exit_status: process::ExitStatus
) -> RunError<'a> {
  use std::os::unix::process::ExitStatusExt;
  match exit_status.signal() {
    Some(signal) => RunError::BacktickSignal{token: token.clone(), signal: signal},
    None => RunError::BacktickUnknownFailure{token: token.clone()},
  }
}

#[cfg(windows)]
fn backtick_error_from_signal<'a>(
  token:       &Token<'a>,
  exit_status: process::ExitStatus
) -> RunError<'a> {
  RunError::BacktickUnknownFailure{token: token.clone()}
}

fn export_env<'a>(
  command: &mut process::Command,
  scope:   &Map<&'a str, String>,
  exports: &Set<&'a str>,
) -> Result<(), RunError<'a>> {
  for name in exports {
    if let Some(value) = scope.get(name) {
      command.env(name, value);
    } else {
      return Err(RunError::InternalError {
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
) -> Result<String, RunError<'a>> {
  let mut cmd = process::Command::new("sh");

  try!(export_env(&mut cmd, scope, exports));

  cmd.arg("-cu")
     .arg(raw);

  cmd.stderr(if quiet {
    process::Stdio::null()
  } else {
    process::Stdio::inherit()
  });

  match cmd.output() {
    Ok(output) => {
      if let Some(code) = output.status.code() {
        if code != 0 {
          return Err(RunError::BacktickCode {
            token: token.clone(),
            code: code,
          });
        }
      } else {
        return Err(backtick_error_from_signal(token, output.status));
      }
      match std::str::from_utf8(&output.stdout) {
        Err(error) => Err(RunError::BacktickUtf8Error{token: token.clone(), utf8_error: error}),
        Ok(utf8) => {
          Ok(if utf8.ends_with('\n') {
            &utf8[0..utf8.len()-1]
          } else if utf8.ends_with("\r\n") {
            &utf8[0..utf8.len()-2]
          } else {
            utf8
          }.to_string())
        }
      }
    }
    Err(error) => Err(RunError::BacktickIoError{token: token.clone(), io_error: error}),
  }
}

impl<'a> Recipe<'a> {
  fn run(
    &self,
    arguments: &[&'a str],
    scope:     &Map<&'a str, String>,
    exports:   &Set<&'a str>,
    options:   &RunOptions,
  ) -> Result<(), RunError<'a>> {
    let argument_map = arguments .iter().enumerate()
      .map(|(i, argument)| (self.parameters[i], *argument)).collect();

    let mut evaluator = Evaluator {
      evaluated:   Map::new(),
      scope:       scope,
      exports:     exports,
      assignments: &Map::new(),
      overrides:   &Map::new(),
      quiet:       options.quiet,
    };

    if self.shebang {
      let mut evaluated_lines = vec![];
      for line in &self.lines {
        evaluated_lines.push(try!(evaluator.evaluate_line(&line, &argument_map)));
      }

      if options.dry_run {
        for line in evaluated_lines {
          warn!("{}", line);
        }
        return Ok(());
      }

      let tmp = try!(
        tempdir::TempDir::new("just")
        .map_err(|error| RunError::TmpdirIoError{recipe: self.name, io_error: error})
      );
      let mut path = tmp.path().to_path_buf();
      path.push(self.name);
      {
        let mut f = try!(
          fs::File::create(&path)
          .map_err(|error| RunError::TmpdirIoError{recipe: self.name, io_error: error})
        );
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
        try!(
          f.write_all(text.as_bytes())
          .map_err(|error| RunError::TmpdirIoError{recipe: self.name, io_error: error})
        );
      }

      // get current permissions
      let mut perms = try!(
        fs::metadata(&path)
        .map_err(|error| RunError::TmpdirIoError{recipe: self.name, io_error: error})
      ).permissions();

      // make the script executable
      let current_mode = perms.mode();
      perms.set_mode(current_mode | 0o100);
      try!(fs::set_permissions(&path, perms)
           .map_err(|error| RunError::TmpdirIoError{recipe: self.name, io_error: error}));

      // run it!
      let mut command = process::Command::new(path);

      try!(export_env(&mut command, scope, exports));

      try!(match command.status() {
        Ok(exit_status) => if let Some(code) = exit_status.code() {
          if code == 0 {
            Ok(())
          } else {
            Err(RunError::Code{recipe: self.name, code: code})
          }
        } else {
          Err(error_from_signal(self.name, exit_status))
        },
        Err(io_error) => Err(RunError::TmpdirIoError{recipe: self.name, io_error: io_error})
      });
    } else {
      for line in &self.lines {
        let evaluated = &try!(evaluator.evaluate_line(&line, &argument_map));
        let mut command = evaluated.as_str();
        let quiet_command = command.starts_with('@');
        if quiet_command {
          command = &command[1..];
        }
        if options.dry_run || !(quiet_command || options.quiet) {
          warn!("{}", command);
        }
        if options.dry_run {
          continue;
        }

        let mut cmd = process::Command::new("sh");

        cmd.arg("-cu").arg(command);

        if options.quiet {
          cmd.stderr(process::Stdio::null());
          cmd.stdout(process::Stdio::null());
        }

        try!(export_env(&mut cmd, scope, exports));

        try!(match cmd.status() {
          Ok(exit_status) => if let Some(code) = exit_status.code() {
            if code == 0 {
              Ok(())
            } else {
              Err(RunError::Code{recipe: self.name, code: code})
            }
          } else {
            Err(error_from_signal(self.name, exit_status))
          },
          Err(io_error) => Err(RunError::IoError{recipe: self.name, io_error: io_error})
        });
      }
    }
    Ok(())
  }
}

impl<'a> Display for Recipe<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "{}", self.name));
    for parameter in &self.parameters {
      try!(write!(f, " {}", parameter));
    }
    try!(write!(f, ":"));
    for dependency in &self.dependencies {
      try!(write!(f, " {}", dependency))
    }

    for (i, pieces) in self.lines.iter().enumerate() {
      if i == 0 {
        try!(writeln!(f, ""));
      }
      for (j, piece) in pieces.iter().enumerate() {
        if j == 0 {
          try!(write!(f, "    "));
        }
        match *piece {
          Fragment::Text{ref text} => try!(write!(f, "{}", text.lexeme)),
          Fragment::Expression{ref expression, ..} => 
            try!(write!(f, "{}{}{}", "{{", expression, "}}")),
        }
      }
      if i + 1 < self.lines.len() {
        try!(write!(f, "\n"));
      }
    }
    Ok(())
  }
}

fn resolve_recipes<'a>(
  recipes:     &Map<&'a str, Recipe<'a>>,
  assignments: &Map<&'a str, Expression<'a>>,
  text:        &'a str,
) -> Result<(), Error<'a>> {
  let mut resolver = Resolver {
    seen:              Set::new(),
    stack:             vec![],
    resolved:          Set::new(),
    recipes:           recipes,
  };
  
  for recipe in recipes.values() {
    try!(resolver.resolve(&recipe));
  }

  for recipe in recipes.values() {
    for line in &recipe.lines {
      for fragment in line {
        if let Fragment::Expression{ref expression, ..} = *fragment {
          for variable in expression.variables() {
            let name = variable.lexeme;
            if !(assignments.contains_key(name) || recipe.parameters.contains(&name)) {
              // There's a borrow issue here that seems too difficult to solve.
              // The error derived from the variable token has too short a lifetime,
              // so we create a new error from its contents, which do live long
              // enough.
              //
              // I suspect the solution here is to give recipes, pieces, and expressions
              // two lifetime parameters instead of one, with one being the lifetime
              // of the struct, and the second being the lifetime of the tokens
              // that it contains
              let error = variable.error(ErrorKind::UndefinedVariable{variable: name});
              return Err(Error {
                text:   text,
                index:  error.index,
                line:   error.line,
                column: error.column,
                width:  error.width,
                kind:   ErrorKind::UndefinedVariable {
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
  fn resolve(&mut self, recipe: &Recipe<'a>) -> Result<(), Error<'a>> {
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
            return Err(dependency_token.error(ErrorKind::CircularRecipeDependency {
              recipe: recipe.name,
              circle: self.stack.iter()
                .skip_while(|name| **name != dependency.name)
                .cloned().collect()
            }));
          }
          return self.resolve(dependency);
        },
        None => return Err(dependency_token.error(ErrorKind::UnknownDependency {
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
) -> Result<(), Error<'a>> {

  let mut resolver = AssignmentResolver {
    assignments:       assignments,
    assignment_tokens: assignment_tokens,
    stack:             vec![],
    seen:              Set::new(),
    evaluated:         Set::new(),
  };

  for name in assignments.keys() {
    try!(resolver.resolve_assignment(name));
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
  fn resolve_assignment(&mut self, name: &'a str) -> Result<(), Error<'a>> {
    if self.evaluated.contains(name) {
      return Ok(());
    }

    self.seen.insert(name);
    self.stack.push(name);

    if let Some(expression) = self.assignments.get(name) {
      try!(self.resolve_expression(expression));
      self.evaluated.insert(name);
    } else {
      return Err(internal_error(format!("attempted to resolve unknown assignment `{}`", name)));
    }
    Ok(())
  }

  fn resolve_expression(&mut self, expression: &Expression<'a>) -> Result<(), Error<'a>> {
    match *expression {
      Expression::Variable{name, ref token} => {
        if self.evaluated.contains(name) {
          return Ok(());
        } else if self.seen.contains(name) {
          let token = &self.assignment_tokens[name];
          self.stack.push(name);
          return Err(token.error(ErrorKind::CircularVariableDependency {
            variable: name,
            circle:   self.stack.clone(),
          }));
        } else if self.assignments.contains_key(name) {
          try!(self.resolve_assignment(name));
        } else {
          return Err(token.error(ErrorKind::UndefinedVariable{variable: name}));
        }
      }
      Expression::Concatination{ref lhs, ref rhs} => {
        try!(self.resolve_expression(lhs));
        try!(self.resolve_expression(rhs));
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
) -> Result<Map<&'a str, String>, RunError<'a>> {
  let mut evaluator = Evaluator {
    assignments: assignments,
    evaluated:   Map::new(),
    exports:     &Set::new(),
    overrides:   overrides,
    quiet:       quiet,
    scope:       &Map::new(),
  };

  for name in assignments.keys() {
    try!(evaluator.evaluate_assignment(name));
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
    arguments: &Map<&str, &str>
  ) -> Result<String, RunError<'a>> {
    let mut evaluated = String::new();
    for fragment in line {
      match *fragment {
        Fragment::Text{ref text} => evaluated += text.lexeme,
        Fragment::Expression{ref expression} => {
          evaluated += &try!(self.evaluate_expression(expression, arguments));
        }
      }
    }
    Ok(evaluated)
  }

  fn evaluate_assignment(&mut self, name: &'a str) -> Result<(), RunError<'a>> {
    if self.evaluated.contains_key(name) {
      return Ok(());
    }

    if let Some(expression) = self.assignments.get(name) {
      if let Some(value) = self.overrides.get(name) {
        self.evaluated.insert(name, value.to_string());
      } else {
        let value = try!(self.evaluate_expression(expression, &Map::new()));
        self.evaluated.insert(name, value);
      }
    } else {
      return Err(RunError::InternalError { 
        message: format!("attempted to evaluated unknown assignment {}", name)
      });
    }

    Ok(())
  }

  fn evaluate_expression(
    &mut self,
    expression: &Expression<'a>,
    arguments: &Map<&str, &str>
  ) -> Result<String, RunError<'a>> {
    Ok(match *expression {
      Expression::Variable{name, ..} => {
        if self.evaluated.contains_key(name) {
          self.evaluated[name].clone()
        } else if self.scope.contains_key(name) {
          self.scope[name].clone()
        } else if self.assignments.contains_key(name) {
          try!(self.evaluate_assignment(name));
          self.evaluated[name].clone()
        } else if arguments.contains_key(name) {
          arguments[name].to_string()
        } else {
          return Err(RunError::InternalError { 
            message: format!("attempted to evaluate undefined variable `{}`", name)
          });
        }
      }
      Expression::String{ref cooked, ..} => cooked.clone(),
      Expression::Backtick{raw, ref token} => {
        try!(run_backtick(raw, token, &self.scope, &self.exports, self.quiet))
      }
      Expression::Concatination{ref lhs, ref rhs} => {
        try!(self.evaluate_expression(lhs, arguments))
          +
        &try!(self.evaluate_expression(rhs, arguments))
      }
    })
  }
}

#[derive(Debug, PartialEq)]
struct Error<'a> {
  text:   &'a str,
  index:  usize,
  line:   usize,
  column: usize,
  width:  Option<usize>,
  kind:   ErrorKind<'a>,
}

#[derive(Debug, PartialEq)]
enum ErrorKind<'a> {
  CircularRecipeDependency{recipe: &'a str, circle: Vec<&'a str>},
  CircularVariableDependency{variable: &'a str, circle: Vec<&'a str>},
  DependencyHasParameters{recipe: &'a str, dependency: &'a str},
  DuplicateDependency{recipe: &'a str, dependency: &'a str},
  DuplicateParameter{recipe: &'a str, parameter: &'a str},
  DuplicateRecipe{recipe: &'a str, first: usize},
  DuplicateVariable{variable: &'a str},
  ExtraLeadingWhitespace,
  InconsistentLeadingWhitespace{expected: &'a str, found: &'a str},
  InternalError{message: String},
  InvalidEscapeSequence{character: char},
  MixedLeadingWhitespace{whitespace: &'a str},
  OuterShebang,
  ParameterShadowsVariable{parameter: &'a str},
  UndefinedVariable{variable: &'a str},
  UnexpectedToken{expected: Vec<TokenKind>, found: TokenKind},
  UnknownDependency{recipe: &'a str, unknown: &'a str},
  UnknownStartOfToken,
  UnterminatedString,
}

fn internal_error(message: String) -> Error<'static> {
  Error {
    text:   "",
    index:  0,
    line:   0,
    column: 0,
    width:  None,
    kind:   ErrorKind::InternalError { message: message }
  }
}

fn show_whitespace(text: &str) -> String {
  text.chars().map(|c| match c { '\t' => 't', ' ' => 's', _ => c }).collect()
}

fn mixed_whitespace(text: &str) -> bool {
  !(text.chars().all(|c| c == ' ') || text.chars().all(|c| c == '\t'))
}

struct And<'a, T: 'a + Display>(&'a [T]);
struct Or <'a, T: 'a + Display>(&'a [T]);

impl<'a, T: Display> Display for And<'a, T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    conjoin(f, self.0, "and")
  }
}

impl<'a, T: Display> Display for Or<'a, T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    conjoin(f, self.0, "or")
  }
}

fn conjoin<T: Display>(
  f:           &mut fmt::Formatter,
  values:      &[T],
  conjunction: &str,
) -> Result<(), fmt::Error> {
    match values.len() {
      0 => {},
      1 => try!(write!(f, "{}", values[0])),
      2 => try!(write!(f, "{} {} {}", values[0], conjunction, values[1])),
      _ => for (i, item) in values.iter().enumerate() {
        try!(write!(f, "{}", item));
        if i == values.len() - 1 {
        } else if i == values.len() - 2 {
          try!(write!(f, ", {} ", conjunction));
        } else {
          try!(write!(f, ", "))
        }
      },
    }
    Ok(())
}

fn write_error_context(
  f:      &mut fmt::Formatter,
  text:   &str,
  index:  usize,
  line:   usize,
  column: usize,
  width:  Option<usize>,
) -> Result<(), fmt::Error> {
  let line_number = line + 1;
  let red = maybe_red(f.alternate());
  match text.lines().nth(line) {
    Some(line) => {
      let line_number_width = line_number.to_string().len();
      try!(write!(f, "{0:1$} |\n", "", line_number_width));
      try!(write!(f, "{} | {}\n", line_number, line));
      try!(write!(f, "{0:1$} |", "", line_number_width));
      try!(write!(f, " {0:1$}{2}{3:^<4$}{5}", "", column,
                  red.prefix(), "", width.unwrap_or(1), red.suffix()));
    },
    None => if index != text.len() {
      try!(write!(f, "internal error: Error has invalid line number: {}", line_number))
    },
  }
  Ok(())
}

fn write_token_error_context(f: &mut fmt::Formatter, token: &Token) -> Result<(), fmt::Error> {
  write_error_context(
    f,
    token.text,
    token.index,
    token.line,
    token.column + token.prefix.len(),
    Some(token.lexeme.len())
  )
}

fn maybe_red(colors: bool) -> ansi_term::Style {
  if colors {
    ansi_term::Style::new().fg(ansi_term::Color::Red)
  } else {
    ansi_term::Style::default()
  }
}

fn maybe_bold(colors: bool) -> ansi_term::Style {
  if colors {
    ansi_term::Style::new().bold()
  } else {
    ansi_term::Style::default()
  }
}

impl<'a> Display for Error<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let red  = maybe_red(f.alternate());
    let bold = maybe_bold(f.alternate());

    try!(write!(f, "{} {}", red.paint("error:"), bold.prefix()));
    
    match self.kind {
      ErrorKind::CircularRecipeDependency{recipe, ref circle} => {
        if circle.len() == 2 {
          try!(write!(f, "recipe `{}` depends on itself", recipe));
        } else {
          try!(writeln!(f, "recipe `{}` has circular dependency `{}`",
                        recipe, circle.join(" -> ")));
        }
      }
      ErrorKind::CircularVariableDependency{variable, ref circle} => {
        if circle.len() == 2 {
          try!(writeln!(f, "variable `{}` depends on its own value: `{}`",
                        variable, circle.join(" -> ")));
        } else {
          try!(writeln!(f, "variable `{}` depends on its own value: `{}`",
                        variable, circle.join(" -> ")));
        }
      }
      ErrorKind::InvalidEscapeSequence{character} => {
        try!(writeln!(f, "`\\{}` is not a valid escape sequence",
                      character.escape_default().collect::<String>()));
      }
      ErrorKind::DuplicateParameter{recipe, parameter} => {
        try!(writeln!(f, "recipe `{}` has duplicate parameter `{}`", recipe, parameter));
      }
      ErrorKind::DuplicateVariable{variable} => {
        try!(writeln!(f, "variable `{}` is has multiple definitions", variable));
      }
      ErrorKind::UnexpectedToken{ref expected, found} => {
        try!(writeln!(f, "expected {} but found {}", Or(expected), found));
      }
      ErrorKind::DuplicateDependency{recipe, dependency} => {
        try!(writeln!(f, "recipe `{}` has duplicate dependency `{}`", recipe, dependency));
      }
      ErrorKind::DuplicateRecipe{recipe, first} => {
        try!(writeln!(f, "recipe `{}` first defined on line {} is redefined on line {}",
                    recipe, first, self.line));
      }
      ErrorKind::DependencyHasParameters{recipe, dependency} => {
        try!(writeln!(f, "recipe `{}` depends on `{}` which requires arguments. \
                      dependencies may not require arguments", recipe, dependency));
      }
      ErrorKind::ParameterShadowsVariable{parameter} => {
        try!(writeln!(f, "parameter `{}` shadows variable of the same name", parameter));
      }
      ErrorKind::MixedLeadingWhitespace{whitespace} => {
        try!(writeln!(f,
          "found a mix of tabs and spaces in leading whitespace: `{}`\n\
          leading whitespace may consist of tabs or spaces, but not both",
          show_whitespace(whitespace)
        ));
      }
      ErrorKind::ExtraLeadingWhitespace => {
        try!(writeln!(f, "recipe line has extra leading whitespace"));
      }
      ErrorKind::InconsistentLeadingWhitespace{expected, found} => {
        try!(writeln!(f,
          "inconsistant leading whitespace: recipe started with `{}` but found line with `{}`:",
          show_whitespace(expected), show_whitespace(found)
        ));
      }
      ErrorKind::OuterShebang => {
        try!(writeln!(f, "a shebang `#!` is reserved syntax outside of recipes"))
      }
      ErrorKind::UnknownDependency{recipe, unknown} => {
        try!(writeln!(f, "recipe `{}` has unknown dependency `{}`", recipe, unknown));
      }
      ErrorKind::UndefinedVariable{variable} => {
        try!(writeln!(f, "variable `{}` not defined", variable));
      }
      ErrorKind::UnknownStartOfToken => {
        try!(writeln!(f, "unknown start of token:"));
      }
      ErrorKind::UnterminatedString => {
        try!(writeln!(f, "unterminated string"));
      }
      ErrorKind::InternalError{ref message} => {
        try!(writeln!(f, "internal error, this may indicate a bug in just: {}\n\
                          consider filing an issue: https://github.com/casey/just/issues/new",
                          message));
      }
    }

    try!(write!(f, "{}", bold.suffix()));

    try!(write_error_context(f, self.text, self.index, self.line, self.column, self.width));

    Ok(())
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
  overrides: Map<&'a str, &'a str>,
  quiet:     bool,
}

impl<'a, 'b> Justfile<'a> where 'a: 'b {
  fn first(&self) -> Option<&'a str> {
    let mut first: Option<&Recipe<'a>> = None;
    for recipe in self.recipes.values() {
      if let Some(first_recipe) = first {
        if recipe.line_number < first_recipe.line_number {
          first = Some(recipe)
        }
      } else {
        first = Some(recipe);
      }
    }
    first.map(|recipe| recipe.name)
  }

  fn count(&self) -> usize {
    self.recipes.len()
  }

  fn recipes(&self) -> Vec<&'a str> {
    self.recipes.keys().cloned().collect()
  }

  fn run(
    &'a self,
    arguments: &[&'a str],
    options:   &RunOptions<'a>,
  ) -> Result<(), RunError<'a>> {
    let unknown_overrides = options.overrides.keys().cloned()
      .filter(|name| !self.assignments.contains_key(name))
      .collect::<Vec<_>>();

    if !unknown_overrides.is_empty() {
      return Err(RunError::UnknownOverrides{overrides: unknown_overrides});
    }

    let scope = try!(evaluate_assignments(&self.assignments, &options.overrides, options.quiet));
    if options.evaluate {
      for (name, value) in scope {
        println!("{} = \"{}\"", name, value);
      }
      return Ok(());
    }

    let mut ran = Set::new();

    for (i, argument) in arguments.iter().enumerate() {
      if let Some(recipe) = self.recipes.get(argument) {
        if !recipe.parameters.is_empty() {
          if i != 0 {
            return Err(RunError::NonLeadingRecipeWithParameters{recipe: recipe.name});
          }
          let rest = &arguments[1..];
          if recipe.parameters.len() != rest.len() {
            return Err(RunError::ArgumentCountMismatch {
              recipe: recipe.name,
              found: rest.len(),
              expected: recipe.parameters.len(),
            });
          }
          try!(self.run_recipe(recipe, rest, &scope, &mut ran, options));
          return Ok(());
        }
      } else {
        break;
      }
    }

    let mut missing = vec![];
    for recipe in arguments {
      if !self.recipes.contains_key(recipe) {
        missing.push(*recipe);
      }
    }
    if !missing.is_empty() {
      return Err(RunError::UnknownRecipes{recipes: missing});
    }
    for recipe in arguments.iter().map(|name| &self.recipes[name]) {
      try!(self.run_recipe(recipe, &[], &scope, &mut ran, options));
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
  ) -> Result<(), RunError> {
    for dependency_name in &recipe.dependencies {
      if !ran.contains(dependency_name) {
        try!(self.run_recipe(&self.recipes[dependency_name], &[], scope, ran, options));
      }
    }
    try!(recipe.run(arguments, &scope, &self.exports, options));
    ran.insert(recipe.name);
    Ok(())
  }

  fn get(&self, name: &str) -> Option<&Recipe<'a>> {
    self.recipes.get(name)
  }
}

impl<'a> Display for Justfile<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let mut items = self.recipes.len() + self.assignments.len();
    for (name, expression) in &self.assignments {
      if self.exports.contains(name) {
        try!(write!(f, "export "));
      }
      try!(write!(f, "{} = {}", name, expression));
      items -= 1;
      if items != 0 {
        try!(write!(f, "\n\n"));
      }
    }
    for recipe in self.recipes.values() {
      try!(write!(f, "{}", recipe));
      items -= 1;
      if items != 0 {
        try!(write!(f, "\n\n"));
      }
    }
    Ok(())
  }
}

#[derive(Debug)]
enum RunError<'a> {
  ArgumentCountMismatch{recipe: &'a str, found: usize, expected: usize},
  Code{recipe: &'a str, code: i32},
  InternalError{message: String},
  IoError{recipe: &'a str, io_error: io::Error},
  NonLeadingRecipeWithParameters{recipe: &'a str},
  Signal{recipe: &'a str, signal: i32},
  TmpdirIoError{recipe: &'a str, io_error: io::Error},
  UnknownFailure{recipe: &'a str},
  UnknownRecipes{recipes: Vec<&'a str>},
  UnknownOverrides{overrides: Vec<&'a str>},
  BacktickCode{token: Token<'a>, code: i32},
  BacktickIoError{token: Token<'a>, io_error: io::Error},
  BacktickSignal{token: Token<'a>, signal: i32},
  BacktickUtf8Error{token: Token<'a>, utf8_error: std::str::Utf8Error},
  BacktickUnknownFailure{token: Token<'a>},
}

impl<'a> Display for RunError<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      RunError::UnknownRecipes{ref recipes} => {
        if recipes.len() == 1 { 
          try!(write!(f, "Justfile does not contain recipe `{}`", recipes[0]));
        } else {
          try!(write!(f, "Justfile does not contain recipes: {}", recipes.join(" ")));
        };
      },
      RunError::UnknownOverrides{ref overrides} => {
        try!(write!(f, "{} set on the command line but not present in justfile",
                    And(overrides)))
      },
      RunError::NonLeadingRecipeWithParameters{recipe} => {
        try!(write!(f, "Recipe `{}` takes arguments and so must be the first and only recipe \
                        specified on the command line", recipe));
      },
      RunError::ArgumentCountMismatch{recipe, found, expected} => {
        try!(write!(f, "Recipe `{}` takes {} argument{}, but {}{} were found",
                    recipe, expected, if expected == 1 { "" } else { "s" }, 
                    if found < expected { "only " } else { "" }, found));
      },
      RunError::Code{recipe, code} => {
        try!(write!(f, "Recipe \"{}\" failed with exit code {}", recipe, code));
      },
      RunError::Signal{recipe, signal} => {
        try!(write!(f, "Recipe \"{}\" wast terminated by signal {}", recipe, signal));
      }
      RunError::UnknownFailure{recipe} => {
        try!(write!(f, "Recipe \"{}\" failed for an unknown reason", recipe));
      },
      RunError::IoError{recipe, ref io_error} => {
        try!(match io_error.kind() {
          io::ErrorKind::NotFound => write!(f,
            "Recipe \"{}\" could not be run because just could not find `sh` the command:\n{}",
            recipe, io_error),
          io::ErrorKind::PermissionDenied => write!(
            f, "Recipe \"{}\" could not be run because just could not run `sh`:\n{}",
            recipe, io_error),
          _ => write!(f, "Recipe \"{}\" could not be run because of an IO error while \
                      launching `sh`:\n{}", recipe, io_error),
        });
      },
      RunError::TmpdirIoError{recipe, ref io_error} =>
        try!(write!(f, "Recipe \"{}\" could not be run because of an IO error while trying \
                    to create a temporary directory or write a file to that directory`:\n{}",
                    recipe, io_error)),
      RunError::BacktickCode{code, ref token} => {
        try!(write!(f, "backtick failed with exit code {}\n", code));
        try!(write_token_error_context(f, token));
      }
      RunError::BacktickSignal{ref token, signal} => {
        try!(write!(f, "backtick was terminated by signal {}", signal));
        try!(write_token_error_context(f, token));
      }
      RunError::BacktickUnknownFailure{ref token} => {
        try!(write!(f, "backtick failed for an uknown reason"));
        try!(write_token_error_context(f, token));
      }
      RunError::BacktickIoError{ref token, ref io_error} => {
        try!(match io_error.kind() {
          io::ErrorKind::NotFound => write!(
            f, "backtick could not be run because just could not find `sh` the command:\n{}",
            io_error),
          io::ErrorKind::PermissionDenied => write!(
            f, "backtick could not be run because just could not run `sh`:\n{}", io_error),
          _ => write!(f, "backtick could not be run because of an IO \
                          error while launching `sh`:\n{}", io_error),
        });
        try!(write_token_error_context(f, token));
      }
      RunError::BacktickUtf8Error{ref token, ref utf8_error} => {
        try!(write!(f, "backtick succeeded but stdout was not utf8: {}", utf8_error));
        try!(write_token_error_context(f, token));
      }
      RunError::InternalError{ref message} => {
        try!(write!(f, "internal error, this may indicate a bug in just: {}
consider filing an issue: https://github.com/casey/just/issues/new", message));
      }
    }

    Ok(())
  }
}

#[derive(Debug, PartialEq, Clone)]
struct Token<'a> {
  index:  usize,
  line:   usize,
  column: usize,
  text:   &'a str,
  prefix: &'a str,
  lexeme: &'a str,
  kind:   TokenKind,
}

impl<'a> Token<'a> {
  fn error(&self, kind: ErrorKind<'a>) -> Error<'a> {
    Error {
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
enum TokenKind {
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
  StringToken,
  RawString,
  Text,
}

impl Display for TokenKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "{}", match *self {
      Backtick           => "backtick",
      Colon              => "\":\"",
      Comment            => "comment",
      Dedent             => "dedent",
      Eof                => "end of file",
      Eol                => "end of line",
      Equals             => "\"=\"",
      Indent             => "indent",
      InterpolationEnd   => "}}",
      InterpolationStart => "{{",
      Line               => "command",
      Name               => "name",
      Plus               => "\"+\"",
      StringToken        => "string",
      RawString          => "raw string",
      Text               => "command text",
    }));
    Ok(())
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

fn tokenize(text: &str) -> Result<Vec<Token>, Error> {
  lazy_static! {
    static ref BACKTICK:                  Regex = token(r"`[^`\n\r]*`"               );
    static ref COLON:                     Regex = token(r":"                         );
    static ref COMMENT:                   Regex = token(r"#([^!].*)?$"               );
    static ref EOF:                       Regex = token(r"(?-m)$"                    );
    static ref EOL:                       Regex = token(r"\n|\r\n"                   );
    static ref EQUALS:                    Regex = token(r"="                         );
    static ref INTERPOLATION_END:         Regex = token(r"[}][}]"                    );
    static ref INTERPOLATION_START_TOKEN: Regex = token(r"[{][{]"                    );
    static ref NAME:                      Regex = token(r"([a-zA-Z_-][a-zA-Z0-9_-]*)");
    static ref PLUS:                      Regex = token(r"[+]"                       );
    static ref STRING:                    Regex = token("\""                         );
    static ref RAW_STRING:                Regex = token(r#"'[^'\r\n]*'"#             );
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
    INDENT.captures(text).map(|captures| captures.at(1).unwrap())
  }

  let mut tokens = vec![];
  let mut rest   = text;
  let mut index  = 0;
  let mut line   = 0;
  let mut column = 0;
  let mut state  = vec![State::Start];

  macro_rules! error {
    ($kind:expr) => {{
      Err(Error {
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
            return error!(ErrorKind::MixedLeadingWhitespace{whitespace: current})
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
            return error!(ErrorKind::InconsistentLeadingWhitespace{
              expected: previous,
              found: current
            });
          }
          None
        }
        // at column 0 in some other state: this should never happen
        (&State::Text, _) | (&State::Interpolation, _) => {
          return error!(ErrorKind::InternalError{
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
      let line = captures.at(0).unwrap();
      if !line.starts_with(indent) {
        return error!(ErrorKind::InternalError{message: "unexpected indent".to_string()});
      }
      state.push(State::Text);
      (&line[0..indent.len()], "", Line)
    } else if let Some(captures) = EOF.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Eof)
    } else if let State::Text = *state.last().unwrap() {
      if let Some(captures) = INTERPOLATION_START.captures(rest) {
        state.push(State::Interpolation);
        ("", captures.at(0).unwrap(), InterpolationStart)
      } else if let Some(captures) = LEADING_TEXT.captures(rest) {
        ("", captures.at(1).unwrap(), Text)
      } else if let Some(captures) = TEXT.captures(rest) {
        ("", captures.at(1).unwrap(), Text)
      } else if let Some(captures) = EOL.captures(rest) {
        state.pop();
        (captures.at(1).unwrap(), captures.at(2).unwrap(), Eol)
      } else {
        return error!(ErrorKind::InternalError{
          message: format!("Could not match token in text state: \"{}\"", rest)
        });
      }
    } else if let Some(captures) = INTERPOLATION_START_TOKEN.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), InterpolationStart)
    } else if let Some(captures) = INTERPOLATION_END.captures(rest) {
      if state.last().unwrap() == &State::Interpolation {
        state.pop();
      }
      (captures.at(1).unwrap(), captures.at(2).unwrap(), InterpolationEnd)
    } else if let Some(captures) = NAME.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Name)
    } else if let Some(captures) = EOL.captures(rest) {
      if state.last().unwrap() == &State::Interpolation {
        return error!(ErrorKind::InternalError { 
          message: "hit EOL while still in interpolation state".to_string()
        });
      }
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Eol)
    } else if let Some(captures) = BACKTICK.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Backtick)
    } else if let Some(captures) = COLON.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Colon)
    } else if let Some(captures) = PLUS.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Plus)
    } else if let Some(captures) = EQUALS.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Equals)
    } else if let Some(captures) = COMMENT.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Comment)
    } else if let Some(captures) = RAW_STRING.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), RawString)
    } else if let Some(captures) = STRING.captures(rest) {
      let prefix = captures.at(1).unwrap();
      let contents = &rest[prefix.len()+1..];
      if contents.is_empty() {
        return error!(ErrorKind::UnterminatedString);
      }
      let mut len = 0;
      let mut escape = false;
      for c in contents.chars() { 
        if c == '\n' || c == '\r' {
          return error!(ErrorKind::UnterminatedString);
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
        return error!(ErrorKind::UnterminatedString);
      }
      (prefix, &rest[start..content_end + 1], StringToken)
    } else if rest.starts_with("#!") {
      return error!(ErrorKind::OuterShebang)
    } else {
      return error!(ErrorKind::UnknownStartOfToken)
    };

    let len = prefix.len() + lexeme.len();

    tokens.push(Token {
      index:  index,
      line:   line,
      column: column,
      prefix: prefix,
      text:   text,
      lexeme: lexeme,
      kind:   kind,
    });

    if len == 0 {
      let last = tokens.last().unwrap();
      match last.kind {
        Eof => {},
        _ => return Err(last.error(ErrorKind::InternalError{
          message: format!("zero length token: {:?}", last)
        })),
      }
    }

    match tokens.last().unwrap().kind {
      Eol => {
        line += 1;
        column = 0;
      },
      Eof => {
        break;
      },
      _ => {
        column += len;
      }
    }

    rest = &rest[len..];
    index += len;
  }

  Ok(tokens)
}

fn parse(text: &str) -> Result<Justfile, Error> {
  let tokens = try!(tokenize(text));
  let filtered: Vec<_> = tokens.into_iter().filter(|token| token.kind != Comment).collect();
  let parser = Parser {
    text:              text,
    tokens:            itertools::put_back(filtered),
    recipes:           Map::<&str, Recipe>::new(),
    assignments:       Map::<&str, Expression>::new(),
    assignment_tokens: Map::<&str, Token>::new(),
    exports:           Set::<&str>::new(),
  };
  let justfile = try!(parser.file());
  Ok(justfile)
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
    if self.peek(Eol) {
      self.accept(Eol);
      None
    } else if self.peek(Eof) {
      None
    } else {
      self.tokens.next()
    }
  }

  fn unexpected_token(&self, found: &Token<'a>, expected: &[TokenKind]) -> Error<'a> {
    found.error(ErrorKind::UnexpectedToken {
      expected: expected.to_vec(),
      found:    found.kind,
    })
  }

  fn recipe(&mut self, name: Token<'a>) -> Result<(), Error<'a>> {
    if let Some(recipe) = self.recipes.get(name.lexeme) {
      return Err(name.error(ErrorKind::DuplicateRecipe {
        recipe: recipe.name,
        first:  recipe.line_number
      }));
    }

    let mut parameters = vec![];
    let mut parameter_tokens = vec![];
    while let Some(parameter) = self.accept(Name) {
      if parameters.contains(&parameter.lexeme) {
        return Err(parameter.error(ErrorKind::DuplicateParameter {
          recipe: name.lexeme, parameter: parameter.lexeme
        }));
      }
      parameters.push(parameter.lexeme);
      parameter_tokens.push(parameter);
    }

    if let Some(token) = self.expect(Colon) {
      // if we haven't accepted any parameters, an equals
      // would have been fine as part of an assignment
      if parameters.is_empty() {
        return Err(self.unexpected_token(&token, &[Name, Colon, Equals]));
      } else {
        return Err(self.unexpected_token(&token, &[Name, Colon]));
      }
    }

    let mut dependencies = vec![];
    let mut dependency_tokens = vec![];
    while let Some(dependency) = self.accept(Name) {
      if dependencies.contains(&dependency.lexeme) {
        return Err(dependency.error(ErrorKind::DuplicateDependency {
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

    let mut lines = vec![];
    let mut shebang = false;

    if self.accepted(Indent) {
      while !self.accepted(Dedent) {
        if self.accepted(Eol) {
          continue;
        }
        if let Some(token) = self.expect(Line) {
          return Err(token.error(ErrorKind::InternalError{
            message: format!("Expected a line but got {}", token.kind)
          }))
        }
        let mut pieces = vec![];

        while !(self.accepted(Eol) || self.peek(Dedent)) {
          if let Some(token) = self.accept(Text) {
            if pieces.is_empty() {
              if lines.is_empty() {
                if token.lexeme.starts_with("#!") {
                  shebang = true;
                }
              } else if !shebang && (token.lexeme.starts_with(' ') || 
                                     token.lexeme.starts_with('\t')) {
                return Err(token.error(ErrorKind::ExtraLeadingWhitespace));
              }
            }
            pieces.push(Fragment::Text{text: token});
          } else if let Some(token) = self.expect(InterpolationStart) {
            return Err(self.unexpected_token(&token, &[Text, InterpolationStart, Eol]));
          } else {
            pieces.push(Fragment::Expression{
              expression: try!(self.expression(true))
            });
            if let Some(token) = self.expect(InterpolationEnd) {
              return Err(self.unexpected_token(&token, &[InterpolationEnd]));
            }
          }
        }

        lines.push(pieces);
      }
    }

    self.recipes.insert(name.lexeme, Recipe {
      line_number:       name.line,
      name:              name.lexeme,
      dependencies:      dependencies,
      dependency_tokens: dependency_tokens,
      parameters:        parameters,
      parameter_tokens:  parameter_tokens,
      lines:             lines,
      shebang:           shebang,
    });

    Ok(())
  }


  fn expression(&mut self, interpolation: bool) -> Result<Expression<'a>, Error<'a>> {
    let first = self.tokens.next().unwrap();
    let lhs = match first.kind {
      Name        => Expression::Variable {name: first.lexeme, token: first},
      Backtick    => Expression::Backtick {
        raw:   &first.lexeme[1..first.lexeme.len()-1],
        token: first
      },
      RawString => {
        let raw = &first.lexeme[1..first.lexeme.len() - 1];
        Expression::String{raw: raw, cooked: raw.to_string()}
      }
      StringToken => {
        let raw = &first.lexeme[1..first.lexeme.len() - 1];
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
              other => return Err(first.error(ErrorKind::InvalidEscapeSequence {
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
        Expression::String{raw: raw, cooked: cooked}
      }
      _ => return Err(self.unexpected_token(&first, &[Name, StringToken])),
    };

    if self.accepted(Plus) {
      let rhs = try!(self.expression(interpolation));
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

  fn assignment(&mut self, name: Token<'a>, export: bool) -> Result<(), Error<'a>> {
    if self.assignments.contains_key(name.lexeme) {
      return Err(name.error(ErrorKind::DuplicateVariable {variable: name.lexeme}));
    }
    if export {
      self.exports.insert(name.lexeme);
    }
    let expression = try!(self.expression(false));
    self.assignments.insert(name.lexeme, expression);
    self.assignment_tokens.insert(name.lexeme, name);
    Ok(())
  }

  fn file(mut self) -> Result<Justfile<'a>, Error<'a>> {
    loop {
      match self.tokens.next() {
        Some(token) => match token.kind {
          Eof => break,
          Eol => continue,
          Name => if token.lexeme == "export" {
            let next = self.tokens.next().unwrap();
            if next.kind == Name && self.accepted(Equals) {
              try!(self.assignment(next, true));
            } else {
              self.tokens.put_back(next);
              try!(self.recipe(token));
            }
          } else if self.accepted(Equals) {
            try!(self.assignment(token, false));
          } else {
            try!(self.recipe(token));
          },
          Comment => return Err(token.error(ErrorKind::InternalError {
            message: "found comment in token stream".to_string()
          })),
          _ => return return Err(self.unexpected_token(&token, &[Name])),
        },
        None => return Err(Error {
          text:   self.text,
          index:  0,
          line:   0,
          column: 0,
          width:  None,
          kind:   ErrorKind::InternalError {
            message: "unexpected end of token stream".to_string()
          }
        }),
      }
    }

    if let Some(token) = self.tokens.next() {
      return Err(token.error(ErrorKind::InternalError{
        message: format!("unexpected token remaining after parsing completed: {:?}", token.kind)
      }))
    }

    try!(resolve_recipes(&self.recipes, &self.assignments, self.text));

    for recipe in self.recipes.values() {
      for parameter in &recipe.parameter_tokens {
        if self.assignments.contains_key(parameter.lexeme) {
          return Err(parameter.error(ErrorKind::ParameterShadowsVariable {
            parameter: parameter.lexeme
          }));
        }
      }

      for dependency in &recipe.dependency_tokens {
        if !self.recipes[dependency.lexeme].parameters.is_empty() {
          return Err(dependency.error(ErrorKind::DependencyHasParameters {
            recipe: recipe.name,
            dependency: dependency.lexeme,
          }));
        }
      }
    }

    try!(resolve_assignments(&self.assignments, &self.assignment_tokens));

    Ok(Justfile {
      recipes:     self.recipes,
      assignments: self.assignments,
      exports:     self.exports,
    })
  }
}
