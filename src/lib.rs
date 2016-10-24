#[cfg(test)]
mod tests;

mod app;

pub use app::app;

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate tempdir;

use std::io::prelude::*;

use std::{fs, fmt, process, io};
use std::collections::{BTreeMap, HashSet};
use std::fmt::Display;
use regex::Regex;

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
  line_number:        usize,
  name:               &'a str,
  lines:              Vec<&'a str>,
  // fragments:          Vec<Vec<Fragment<'a>>>,
  // variables:          BTreeSet<&'a str>,
  dependencies:       Vec<&'a str>,
  dependency_tokens:  Vec<Token<'a>>,
  arguments:          Vec<&'a str>,
  argument_tokens:    Vec<Token<'a>>,
  shebang:            bool,
}

/*
enum Fragment<'a> {
  Text{text: &'a str},
  Variable{name: &'a str},
}
*/

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

impl<'a> Recipe<'a> {
  fn run(&self) -> Result<(), RunError<'a>> {
    if self.shebang {
      let tmp = try!(
        tempdir::TempDir::new("j")
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
        text += self.lines[0];
        text += "\n";
        // add blank lines so that lines in the generated script
        // have the same line number as the corresponding lines
        // in the justfile
        for _ in 1..(self.line_number + 2) {
          text += "\n"
        }
        for line in &self.lines[1..] {
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
      try!(fs::set_permissions(&path, perms).map_err(|error| RunError::TmpdirIoError{recipe: self.name, io_error: error}));

      // run it!
      let status = process::Command::new(path).status();
      try!(match status {
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
      for command in &self.lines {
        let mut command = *command;
        if !command.starts_with('@') {
          warn!("{}", command);
        } else {
          command = &command[1..]; 
        }
        let status = process::Command::new("sh")
          .arg("-cu")
          .arg(command)
          .status();
        try!(match status {
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
    for argument in &self.arguments {
      try!(write!(f, " {}", argument));
    }
    try!(write!(f, ":"));
    for dependency in &self.dependencies {
      try!(write!(f, " {}", dependency))
    }
    for (i, line) in self.lines.iter().enumerate() {
      if i == 0 {
        try!(writeln!(f, ""));
      }
      try!(write!(f, "    {}", line));
      if i + 1 < self.lines.len() {
        try!(writeln!(f, ""));
      }
    }
    Ok(())
  }
}

fn resolve<'a>(
  recipes:  &BTreeMap<&str, Recipe<'a>>,
  resolved: &mut HashSet<&'a str>,
  seen:     &mut HashSet<&'a str>,
  stack:    &mut Vec<&'a str>,
  recipe:   &Recipe<'a>,
) -> Result<(), Error<'a>> {
  if resolved.contains(recipe.name) {
    return Ok(())
  }
  stack.push(recipe.name);
  seen.insert(recipe.name);
  for dependency_token in &recipe.dependency_tokens {
    match recipes.get(dependency_token.lexeme) {
      Some(dependency) => if !resolved.contains(dependency.name) {
        if seen.contains(dependency.name) {
          let first = stack[0];
          stack.push(first);
          return Err(dependency_token.error(ErrorKind::CircularDependency {
            recipe: recipe.name,
            circle: stack.iter()
              .skip_while(|name| **name != dependency.name)
              .cloned().collect()
          }));
        }
        return resolve(recipes, resolved, seen, stack, dependency);
      },
      None => return Err(dependency_token.error(ErrorKind::UnknownDependency {
        recipe:  recipe.name,
        unknown: dependency_token.lexeme
      })),
    }
  }
  resolved.insert(recipe.name);
  stack.pop();
  Ok(())
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
  BadName{name: &'a str},
  CircularDependency{recipe: &'a str, circle: Vec<&'a str>},
  DuplicateDependency{recipe: &'a str, dependency: &'a str},
  DuplicateArgument{recipe: &'a str, argument: &'a str},
  DuplicateRecipe{recipe: &'a str, first: usize},
  MixedLeadingWhitespace{whitespace: &'a str},
  ExtraLeadingWhitespace,
  InconsistentLeadingWhitespace{expected: &'a str, found: &'a str},
  OuterShebang,
  AssignmentUnimplemented,
  UnknownDependency{recipe: &'a str, unknown: &'a str},
  UnknownStartOfToken,
  UnexpectedToken{expected: Vec<TokenKind>, found: TokenKind},
  InternalError{message: String},
}

fn show_whitespace(text: &str) -> String {
  text.chars().map(|c| match c { '\t' => 't', ' ' => 's', _ => c }).collect()
}

fn mixed_whitespace(text: &str) -> bool {
  !(text.chars().all(|c| c == ' ') || text.chars().all(|c| c == '\t'))
}

struct Or<'a, T: 'a + Display>(&'a [T]);

impl<'a, T: Display> Display for Or<'a, T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self.0.len() {
      0 => {},
      1 => try!(write!(f, "{}", self.0[0])),
      2 => try!(write!(f, "{} or {}", self.0[0], self.0[1])),
      _ => for (i, item) in self.0.iter().enumerate() {
        try!(write!(f, "{}", item));
        if i == self.0.len() - 1 {
        } else if i == self.0.len() - 2 {
          try!(write!(f, ", or "));
        } else {
          try!(write!(f, ", "))
        }
      },
    }
    Ok(())
  }
}

impl<'a> Display for Error<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "justfile:{}: ", self.line));

    match self.kind {
      ErrorKind::BadName{name} => {
         try!(writeln!(f, "name did not match /[a-z](-?[a-z0-9])*/: {}", name));
      }
      ErrorKind::CircularDependency{recipe, ref circle} => {
        try!(write!(f, "recipe {} has circular dependency: {}", recipe, circle.join(" -> ")));
        return Ok(());
      }
      ErrorKind::DuplicateArgument{recipe, argument} => {
        try!(writeln!(f, "recipe {} has duplicate argument: {}", recipe, argument));
      }
      ErrorKind::UnexpectedToken{ref expected, found} => {
        try!(writeln!(f, "expected {} but found {}", Or(expected), found));
      }
      ErrorKind::DuplicateDependency{recipe, dependency} => {
        try!(writeln!(f, "recipe {} has duplicate dependency: {}", recipe, dependency));
      }
      ErrorKind::DuplicateRecipe{recipe, first} => {
        try!(write!(f, "duplicate recipe: {} appears on lines {} and {}", 
                    recipe, first, self.line));
        return Ok(());
      }
      ErrorKind::MixedLeadingWhitespace{whitespace} => {
        try!(writeln!(f,
          "found a mix of tabs and spaces in leading whitespace: {}\n leading whitespace may consist of tabs or spaces, but not both",
          show_whitespace(whitespace)
        ));
      }
      ErrorKind::ExtraLeadingWhitespace => {
        try!(writeln!(f, "recipe line has extra leading whitespace"));
      }
      ErrorKind::AssignmentUnimplemented => {
        try!(writeln!(f, "variable assignment is not yet implemented"));
      }
      ErrorKind::InconsistentLeadingWhitespace{expected, found} => {
        try!(writeln!(f,
          "inconsistant leading whitespace: recipe started with \"{}\" but found line with \"{}\":",
          show_whitespace(expected), show_whitespace(found)
        ));
      }
      ErrorKind::OuterShebang => {
        try!(writeln!(f, "a shebang \"#!\" is reserved syntax outside of recipes"))
      }
      ErrorKind::UnknownDependency{recipe, unknown} => {
        try!(writeln!(f, "recipe {} has unknown dependency {}", recipe, unknown));
      }
      ErrorKind::UnknownStartOfToken => {
        try!(writeln!(f, "uknown start of token:"));
      }
      ErrorKind::InternalError{ref message} => {
        try!(writeln!(f, "internal error, this may indicate a bug in j: {}\n consider filing an issue: https://github.com/casey/j/issues/new", message));
      }
    }

    match self.text.lines().nth(self.line) {
      Some(line) => try!(write!(f, "{}", line)),
      None => if self.index != self.text.len() {
        try!(write!(f, "internal error: Error has invalid line number: {}", self.line))
      },
    };

    Ok(())
  }
}

struct Justfile<'a> {
  recipes: BTreeMap<&'a str, Recipe<'a>>,
}

impl<'a> Justfile<'a> {
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

  fn run_recipe(&self, recipe: &Recipe<'a>, ran: &mut HashSet<&'a str>) -> Result<(), RunError> {
    for dependency_name in &recipe.dependencies {
      if !ran.contains(dependency_name) {
        try!(self.run_recipe(&self.recipes[dependency_name], ran));
      }
    }
    try!(recipe.run());
    ran.insert(recipe.name);
    Ok(())
  }

  fn run<'b>(&'a self, names: &[&'b str]) -> Result<(), RunError<'b>> 
    where 'a: 'b
  {
    let mut missing = vec![];
    for recipe in names {
      if !self.recipes.contains_key(recipe) {
        missing.push(*recipe);
      }
    }
    if !missing.is_empty() {
      return Err(RunError::UnknownRecipes{recipes: missing});
    }
    let recipes = names.iter().map(|name| self.recipes.get(name).unwrap()).collect::<Vec<_>>();
    let mut ran = HashSet::new();
    for recipe in recipes {
      try!(self.run_recipe(recipe, &mut ran));
    }
    Ok(())
  }

  fn get(&self, name: &str) -> Option<&Recipe<'a>> {
    self.recipes.get(name)
  }
}

#[derive(Debug)]
enum RunError<'a> {
  UnknownRecipes{recipes: Vec<&'a str>},
  Signal{recipe: &'a str, signal: i32},
  Code{recipe: &'a str, code: i32},
  UnknownFailure{recipe: &'a str},
  IoError{recipe: &'a str, io_error: io::Error},
  TmpdirIoError{recipe: &'a str, io_error: io::Error},
}

impl<'a> Display for RunError<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      RunError::UnknownRecipes{ref recipes} => {
        if recipes.len() == 1 { 
          try!(write!(f, "Justfile does not contain recipe: {}", recipes[0]));
        } else {
          try!(write!(f, "Justfile does not contain recipes: {}", recipes.join(" ")));
        };
      },
      RunError::Code{recipe, code} => {
        try!(write!(f, "Recipe \"{}\" failed with code {}", recipe, code));
      },
      RunError::Signal{recipe, signal} => {
        try!(write!(f, "Recipe \"{}\" wast terminated by signal {}", recipe, signal));
      }
      RunError::UnknownFailure{recipe} => {
        try!(write!(f, "Recipe \"{}\" failed for an unknown reason", recipe));
      },
      RunError::IoError{recipe, ref io_error} => {
        try!(match io_error.kind() {
          io::ErrorKind::NotFound => write!(f, "Recipe \"{}\" could not be run because j could not find `sh` the command:\n{}", recipe, io_error),
          io::ErrorKind::PermissionDenied => write!(f, "Recipe \"{}\" could not be run because j could not run `sh`:\n{}", recipe, io_error),
          _ => write!(f, "Recipe \"{}\" could not be run because of an IO error while launching the `sh`:\n{}", recipe, io_error),
        });
      },
      RunError::TmpdirIoError{recipe, ref io_error} =>
        try!(write!(f, "Recipe \"{}\" could not be run because of an IO error while trying to create a temporary directory or write a file to that directory`:\n{}", recipe, io_error)),
    }
    Ok(())
  }
}

#[derive(Debug, PartialEq)]
struct Token<'a> {
  index:  usize,
  line:   usize,
  column: usize,
  text:   &'a str,
  prefix: &'a str,
  lexeme: &'a str,
  class:  TokenKind,
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
  Name,
  Colon,
  Equals,
  Comment,
  Line,
  Indent,
  Dedent,
  Eol,
  Eof,
}

impl Display for TokenKind {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "{}", match *self {
      Name    => "name",
      Colon   => "\":\"",
      Equals  => "\"=\"",
      Comment => "comment",
      Line    => "command",
      Indent  => "indent",
      Dedent  => "dedent",
      Eol     => "end of line",
      Eof     => "end of file",
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
    static ref EOF:       Regex = token(r"(?-m)$"                );
    static ref NAME:      Regex = token(r"([a-zA-Z0-9_-]+)"      );
    static ref COLON:     Regex = token(r":"                     );
    static ref EQUALS:    Regex = token(r"="                     );
    static ref COMMENT:   Regex = token(r"#([^!].*)?$"           );
    static ref EOL:       Regex = token(r"\n|\r\n"               );
    static ref LINE:      Regex = re(r"^(?m)[ \t]+[^ \t\n\r].*$");
    static ref INDENT:    Regex = re(r"^([ \t]*)[^ \t\n\r]"     );
  }

  fn indentation(text: &str) -> Option<&str> {
    INDENT.captures(text).map(|captures| captures.at(1).unwrap())
  }

  let mut tokens               = vec![];
  let mut rest                 = text;
  let mut index                = 0;
  let mut line                 = 0;
  let mut column               = 0;
  let mut indent: Option<&str> = None;

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
      if let Some(class) = match (indent, indentation(rest)) {
        // ignore: was no indentation and there still isn't
        //         or current line is blank
        (None, Some("")) | (_, None) => {
          None
        }
        // indent: was no indentation, now there is
        (None, Some(current)) => {
          if mixed_whitespace(current) {
            return error!(ErrorKind::MixedLeadingWhitespace{whitespace: current})
          }
          indent = Some(current);
          Some(Indent)
        }
        // dedent: there was indentation and now there isn't
        (Some(_), Some("")) => {
          indent = None;
          Some(Dedent)
        }
        // was indentation and still is, check if the new indentation matches
        (Some(previous), Some(current)) => {
          if !current.starts_with(previous) {
            return error!(ErrorKind::InconsistentLeadingWhitespace{
              expected: previous,
              found: current
            });
          }
          None
        }
      } {
        tokens.push(Token {
          index:  index,
          line:   line,
          column: column,
          text:   text,
          prefix: "",
          lexeme: "",
          class:  class,
        });
      }
    }

    // insert a dedent if we're indented and we hit the end of the file
    if indent.is_some() && EOF.is_match(rest) {
      tokens.push(Token {
        index:  index,
        line:   line,
        column: column,
        text:   text,
        prefix: "",
        lexeme: "",
        class:  Dedent,
      });
    }

    let (prefix, lexeme, class) = 
    if let (0, Some(indent), Some(captures)) = (column, indent, LINE.captures(rest)) {
      let line = captures.at(0).unwrap();
      if !line.starts_with(indent) {
        return error!(ErrorKind::InternalError{message: "unexpected indent".to_string()});
      }
      let (prefix, lexeme) = line.split_at(indent.len());
      (prefix, lexeme, Line)
    } else if let Some(captures) = NAME.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Name)
    } else if let Some(captures) = EOL.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Eol)
    } else if let Some(captures) = EOF.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Eof)
    } else if let Some(captures) = COLON.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Colon)
    } else if let Some(captures) = EQUALS.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Equals)
    } else if let Some(captures) = COMMENT.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Comment)
    } else {
      return if rest.starts_with("#!") {
        error!(ErrorKind::OuterShebang)
      } else {
        error!(ErrorKind::UnknownStartOfToken)
      };
    };

    let len = prefix.len() + lexeme.len();

    tokens.push(Token {
      index:  index,
      line:   line,
      column: column,
      prefix: prefix,
      text:   text,
      lexeme: lexeme,
      class:  class,
    });

    match tokens.last().unwrap().class {
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
  let filtered: Vec<_> = tokens.into_iter().filter(|token| token.class != Comment).collect();
  if let Some(token) = filtered.iter().find(|token| {
    lazy_static! {
      static ref GOOD_NAME: Regex = re("^[a-z](-?[a-z0-9])*$");
    }
    token.class == Name && !GOOD_NAME.is_match(token.lexeme)
  }) {
    return Err(token.error(ErrorKind::BadName{name: token.lexeme}));
  }

  let parser = Parser{
    text: text,
    tokens: filtered.into_iter().peekable()
  };
  let justfile = try!(parser.file());
  Ok(justfile)
}

struct Parser<'a> {
  text:   &'a str,
  tokens: std::iter::Peekable<std::vec::IntoIter<Token<'a>>>
}

impl<'a> Parser<'a> {
  fn peek(&mut self, class: TokenKind) -> bool {
    self.tokens.peek().unwrap().class == class
  }

  fn accept(&mut self, class: TokenKind) -> Option<Token<'a>> {
    if self.peek(class) {
      self.tokens.next()
    } else {
      None
    }
  }

  fn accepted(&mut self, class: TokenKind) -> bool {
    self.accept(class).is_some()
  }

  fn expect(&mut self, class: TokenKind) -> Option<Token<'a>> {
    if self.peek(class) {
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

  fn recipe(&mut self, name: &'a str, line_number: usize) -> Result<Recipe<'a>, Error<'a>> {
    let mut arguments = vec![];
    let mut argument_tokens = vec![];
    while let Some(argument) = self.accept(Name) {
      if arguments.contains(&argument.lexeme) {
        return Err(argument.error(ErrorKind::DuplicateArgument{
          recipe: name, argument: argument.lexeme
        }));
      }
      arguments.push(argument.lexeme);
      argument_tokens.push(argument);
    }

    if let Some(token) = self.expect(Colon) {
      return Err(self.unexpected_token(&token, &[Name, Colon]));
    }

    let mut dependencies = vec![];
    let mut dependency_tokens = vec![];
    while let Some(dependency) = self.accept(Name) {
      if dependencies.contains(&dependency.lexeme) {
        return Err(dependency.error(ErrorKind::DuplicateDependency {
          recipe:     name,
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
      while !self.peek(Dedent) {
        if let Some(line) = self.accept(Line) {
          if lines.is_empty() {
            if line.lexeme.starts_with("#!") {
              shebang = true;
            }
          } else if !shebang && (line.lexeme.starts_with(' ') || line.lexeme.starts_with('\t')) {
            return Err(line.error(ErrorKind::ExtraLeadingWhitespace));
          }

          lines.push(line.lexeme);
          if !self.peek(Dedent) {
            if let Some(token) = self.expect_eol() {
              return Err(self.unexpected_token(&token, &[Eol]));
            }
          }
        } else if let Some(_) = self.accept(Eol) {
        } else {
          let token = self.tokens.next().unwrap();
          return Err(self.unexpected_token(&token, &[Line, Eol]));
        }
      }

      if let Some(token) = self.expect(Dedent) {
        return Err(self.unexpected_token(&token, &[Dedent]));
      }
    }

    Ok(Recipe {
      line_number:       line_number,
      name:              name,
      dependencies:      dependencies,
      dependency_tokens: dependency_tokens,
      arguments:         arguments,
      argument_tokens:   argument_tokens,
      lines:             lines,
      shebang:           shebang,
    })
  }

  fn unexpected_token(&self, found: &Token<'a>, expected: &[TokenKind]) -> Error<'a> {
    found.error(ErrorKind::UnexpectedToken {
      expected: expected.to_vec(),
      found:    found.class,
    })
  }

  fn file(mut self) -> Result<Justfile<'a>, Error<'a>> {
    let mut recipes = BTreeMap::<&str, Recipe>::new();

    loop {
      match self.tokens.next() {
        Some(token) => match token.class {
          Eof => break,
          Eol => continue,
          Name => if let Some(equals) = self.accept(Equals) {
            return Err(equals.error(ErrorKind::AssignmentUnimplemented));
          } else {
            if let Some(recipe) = recipes.remove(token.lexeme) {
              return Err(token.error(ErrorKind::DuplicateRecipe {
                recipe: recipe.name,
                first:  recipe.line_number
              }));
            }
            recipes.insert(token.lexeme, try!(self.recipe(token.lexeme, token.line)));
          },
          Comment => return Err(token.error(ErrorKind::InternalError {
            message: "found comment in token stream".to_string()
          })),
          _ => return Err(token.error(ErrorKind::InternalError {
            message: format!("unhandled token class: {:?}", token.class)
          })),
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
        message: format!("unexpected token remaining after parsing completed: {:?}", token.class)
      }))
    }

    let mut resolved = HashSet::new();
    let mut seen     = HashSet::new();
    let mut stack    = vec![];

    for recipe in recipes.values() {
      try!(resolve(&recipes, &mut resolved, &mut seen, &mut stack, &recipe));
    }

    Ok(Justfile{recipes: recipes})
  }
}
