#[cfg(test)]
mod tests;

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

pub trait Slurp {
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

pub struct Recipe<'a> {
  line_number:        usize,
  label:              &'a str,
  name:               &'a str,
  // leading_whitespace: &'a str,
  lines:              Vec<&'a str>,
  // fragments:          Vec<Vec<Fragment<'a>>>,
  // variables:          BTreeSet<&'a str>,
  dependencies:       Vec<&'a str>,
  // arguments:          Vec<&'a str>,
  shebang:            bool,
}

/*
enum Fragment<'a> {
  Text{text: &'a str},
  Variable{name: &'a str},
}
*/

impl<'a> Display for Recipe<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(writeln!(f, "{}", self.label));
    for (i, line) in self.lines.iter().enumerate() {
      if i + 1 < self.lines.len() {
        try!(writeln!(f, "    {}", line));
      } {
        try!(write!(f, "    {}", line));
      }
    }
    Ok(())
  }
}

#[cfg(unix)]
fn error_from_signal<'a>(recipe: &'a str, exit_status: process::ExitStatus) -> RunError<'a> {
  use std::os::unix::process::ExitStatusExt;
  match exit_status.signal() {
    Some(signal) => RunError::Signal{recipe: recipe, signal: signal},
    None => RunError::UnknownFailure{recipe: recipe},
  }
}

#[cfg(windows)]
fn error_from_signal<'a>(recipe: &'a str, exit_status: process::ExitStatus) -> RunError<'a> {
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
        if !command.starts_with("@") {
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

/*
fn resolve<'a>(
  text:     &'a str,
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
  for dependency_name in &recipe.dependencies {
    match recipes.get(dependency_name) {
      Some(dependency) => if !resolved.contains(dependency.name) {
        if seen.contains(dependency.name) {
          let first = stack[0];
          stack.push(first);
          return Err(error(text, recipe.line_number, ErrorKind::CircularDependency {
            circle: stack.iter()
              .skip_while(|name| **name != dependency.name)
              .cloned().collect()
          }));
        }
        return resolve(text, recipes, resolved, seen, stack, dependency);
      },
      None => return Err(error(text, recipe.line_number, ErrorKind::UnknownDependency {
        name:    recipe.name,
        unknown: dependency_name
      })),
    }
  }
  resolved.insert(recipe.name);
  stack.pop();
  Ok(())
}
*/

#[derive(Debug, PartialEq)]
pub struct Error<'a> {
  text:   &'a str,
  index:  usize,
  line:   usize,
  column: usize,
  kind:   ErrorKind<'a>,
}

#[derive(Debug, PartialEq)]
enum ErrorKind<'a> {
  // BadRecipeName{name: &'a str},
  // CircularDependency{circle: Vec<&'a str>},
  // DuplicateDependency{name: &'a str},
  // DuplicateArgument{recipe: &'a str, argument: &'a str},
  // DuplicateRecipe{first: usize, name: &'a str},
  // TabAfterSpace{whitespace: &'a str},
  // MixedLeadingWhitespace{whitespace: &'a str},
  // ExtraLeadingWhitespace,
  InconsistentLeadingWhitespace{expected: &'a str, found: &'a str},
  OuterShebang,
  // NonLeadingShebang{recipe: &'a str},
  // UnknownDependency{name: &'a str, unknown: &'a str},
  // Unparsable,
  // UnparsableDependencies,
  UnknownStartOfToken,
  InternalError{message: String},
}

// fn error<'a>(text: &'a str, line: usize, kind: ErrorKind<'a>) 
//   -> Error<'a>
// {
//   Error {
//     text: text,
//     line: line,
//     kind: kind,
//   }
// }

fn show_whitespace(text: &str) -> String {
  text.chars().map(|c| match c { '\t' => 't', ' ' => 's', _ => c }).collect()
}

/*
fn mixed(text: &str) -> bool {
  !(text.chars().all(|c| c == ' ') || text.chars().all(|c| c == '\t'))
}
*/

/*
fn tab_after_space(text: &str) -> bool {
  let mut space = false;
  for c in text.chars() {
    match c {
      ' ' => space = true,
      '\t' => if space {
        return true;
      },
      _ => {},
    }
  }
  return false;
}
*/

impl<'a> Display for Error<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "justfile:{}: ", self.line));

    match self.kind {
      // ErrorKind::BadRecipeName{name} => {
      //   try!(writeln!(f, "recipe name does not match /[a-z](-[a-z]|[a-z])*/: {}", name));
      // }
      // ErrorKind::CircularDependency{ref circle} => {
      //   try!(write!(f, "circular dependency: {}", circle.join(" -> ")));
      //   return Ok(());
      // }
      // ErrorKind::DuplicateArgument{recipe, argument} => {
      //  try!(writeln!(f, "recipe {} has duplicate argument: {}", recipe, argument));
      //}
      // ErrorKind::DuplicateDependency{name} => {
      //   try!(writeln!(f, "duplicate dependency: {}", name));
      // }
      // ErrorKind::DuplicateRecipe{first, name} => {
      //   try!(write!(f, "duplicate recipe: {} appears on lines {} and {}", 
      //               name, first, self.line));
      //   return Ok(());
      // }
      // ErrorKind::TabAfterSpace{whitespace} => {
      //   try!(writeln!(f, "found tab after space: {}", show_whitespace(whitespace)));
      // }
      // ErrorKind::MixedLeadingWhitespace{whitespace} => {
      //   try!(writeln!(f,
      //     "inconsistant leading whitespace: recipe started with {}:",
      //     show_whitespace(whitespace)
      //   ));
      // }
      // ErrorKind::ExtraLeadingWhitespace => {
      //   try!(writeln!(f, "line has extra leading whitespace"));
      // }
      ErrorKind::InconsistentLeadingWhitespace{expected, found} => {
        try!(writeln!(f,
          "inconsistant leading whitespace: recipe started with \"{}\" but found line with \"{}\":",
          show_whitespace(expected), show_whitespace(found)
        ));
      }
      ErrorKind::OuterShebang => {
        try!(writeln!(f, "a shebang \"#!\" is reserved syntax outside of recipes"))
      }
      // ErrorKind::NonLeadingShebang{..} => {
      //  try!(writeln!(f, "a shebang \"#!\" may only appear on the first line of a recipe"))
      //}
      // ErrorKind::UnknownDependency{name, unknown} => {
      //   try!(writeln!(f, "recipe {} has unknown dependency {}", name, unknown));
      // }
      // ErrorKind::Unparsable => {
      //   try!(writeln!(f, "could not parse line:"));
      // }
      // ErrorKind::UnparsableDependencies => {
      //   try!(writeln!(f, "could not parse dependencies:"));
      // }
      ErrorKind::UnknownStartOfToken => {
        try!(writeln!(f, "uknown start of token:"));
      }
      ErrorKind::InternalError{ref message} => {
        try!(writeln!(f, "internal error, this may indicate a bug in j: {}\n consider filing an issue: https://github.com/casey/j/issues/new", message));
      }
    }

    match self.text.lines().nth(self.line) {
      Some(line) => try!(write!(f, "{}", line)),
      None => try!(write!(f, "internal error: Error has invalid line number: {}", self.line)),
    };

    Ok(())
  }
}

pub struct Justfile<'a> {
  recipes: BTreeMap<&'a str, Recipe<'a>>,
}

impl<'a> Justfile<'a> {
  pub fn first(&self) -> Option<&'a str> {
    let mut first: Option<&Recipe<'a>> = None;
    for (_, recipe) in self.recipes.iter() {
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

  pub fn count(&self) -> usize {
    self.recipes.len()
  }

  pub fn recipes(&self) -> Vec<&'a str> {
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

  pub fn run<'b>(&'a self, names: &[&'b str]) -> Result<(), RunError<'b>> 
    where 'a: 'b
  {
    let mut missing = vec![];
    for recipe in names {
      if !self.recipes.contains_key(recipe) {
        missing.push(*recipe);
      }
    }
    if missing.len() > 0 {
      return Err(RunError::UnknownRecipes{recipes: missing});
    }
    let recipes = names.iter().map(|name| self.recipes.get(name).unwrap()).collect::<Vec<_>>();
    let mut ran = HashSet::new();
    for recipe in recipes {
      try!(self.run_recipe(recipe, &mut ran));
    }
    Ok(())
  }

  pub fn get(&self, name: &str) -> Option<&Recipe<'a>> {
    self.recipes.get(name)
  }
}

#[derive(Debug)]
pub enum RunError<'a> {
  UnknownRecipes{recipes: Vec<&'a str>},
  Signal{recipe: &'a str, signal: i32},
  Code{recipe: &'a str, code: i32},
  UnknownFailure{recipe: &'a str},
  IoError{recipe: &'a str, io_error: io::Error},
  TmpdirIoError{recipe: &'a str, io_error: io::Error},
}

impl<'a> Display for RunError<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      &RunError::UnknownRecipes{ref recipes} => {
        if recipes.len() == 1 { 
          try!(write!(f, "Justfile does not contain recipe: {}", recipes[0]));
        } else {
          try!(write!(f, "Justfile does not contain recipes: {}", recipes.join(" ")));
        };
      },
      &RunError::Code{recipe, code} => {
        try!(write!(f, "Recipe \"{}\" failed with code {}", recipe, code));
      },
      &RunError::Signal{recipe, signal} => {
        try!(write!(f, "Recipe \"{}\" wast terminated by signal {}", recipe, signal));
      }
      &RunError::UnknownFailure{recipe} => {
        try!(write!(f, "Recipe \"{}\" failed for an unknown reason", recipe));
      },
      &RunError::IoError{recipe, ref io_error} => {
        try!(match io_error.kind() {
          io::ErrorKind::NotFound => write!(f, "Recipe \"{}\" could not be run because j could not find `sh` the command:\n{}", recipe, io_error),
          io::ErrorKind::PermissionDenied => write!(f, "Recipe \"{}\" could not be run because j could not run `sh`:\n{}", recipe, io_error),
          _ => write!(f, "Recipe \"{}\" could not be run because of an IO error while launching the `sh`:\n{}", recipe, io_error),
        });
      },
      &RunError::TmpdirIoError{recipe, ref io_error} =>
        try!(write!(f, "Recipe \"{}\" could not be run because of an IO error while trying to create a temporary directory or write a file to that directory`:\n{}", recipe, io_error)),
    }
    Ok(())
  }
}

struct Token<'a> {
  index:  usize,
  line:   usize,
  column: usize,
  prefix: &'a str,
  lexeme: &'a str,
  class:  TokenClass,
}

impl<'a> Token<'a> {
  fn error(&self, text: &'a str, kind: ErrorKind<'a>) -> Error<'a> {
    Error {
      text:   text,
      index:  self.index,
      line:   self.line,
      column: self.column,
      kind:   kind,
    }
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum TokenClass {
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

use TokenClass::*;

fn token(pattern: &str) -> Regex {
  let mut s = String::new();
  s += r"^(?m)([ \t]*)(";
  s += pattern;
  s += ")";
  re(&s)
}

fn tokenize(text: &str) -> Result<Vec<Token>, Error> {
  lazy_static! {
    static ref EOF:     Regex = token(r"(?-m)$"                );
    static ref NAME:    Regex = token(r"[a-z]((_|-)?[a-z0-9])*");
    static ref COLON:   Regex = token(r":"                     );
    static ref EQUALS:  Regex = token(r"="                     );
    static ref COMMENT: Regex = token(r"#([^!].*)?$"           );
    static ref EOL:     Regex = token(r"\n|\r\n"               );
    static ref LINE:    Regex = re(r"^(?m)[ \t]+[^ \t\n\r].*$");
    static ref INDENT:  Regex = re(r"^([ \t]*)[^ \t\n\r]"     );
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
        kind:   $kind,
      })
    }};
  }

  loop {
    if column == 0 {
      if let Some(class) = match (indent, indentation(rest)) {
        // ignore: was no indentation and there still isn't
        (None, Some("")) => {
          None
        }
        // ignore: current line is blank
        (_, None) => {
          None
        }
        // indent: was no indentation, now there is
        (None, Some(current @ _)) => {
          // check mixed leading whitespace
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
          // check tabs after spaces
        }
      } {
        tokens.push(Token {
          index:  index,
          line:   line,
          column: column,
          prefix: "",
          lexeme: "",
          class:  class,
        });
      }
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

pub fn parse<'a>(text: &'a str) -> Result<Justfile, Error> {
  let tokens = try!(tokenize(text));
  let filtered: Vec<_> = tokens.into_iter().filter(|t| t.class != Comment).collect();
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
  /*
  fn accept(&mut self, class: TokenClass) -> Option<Token<'a>> {
    if self.peek(class) {
      self.tokens.next()
    } else {
      None
    }
  }

  fn accepted(&mut self, class: TokenClass) -> bool {
    self.accept(class).is_some()
  }

  fn peek(&mut self, class: TokenClass) -> bool {
    self.tokens.peek().unwrap().class == class
  }
  */

  /*

  fn expect(&mut self, class: TokenClass) {
    if !self.accepted(class) {
      panic!("we fucked");
    }
  }
  */

  /*


  // fn accept(&mut self) -> Result<Token<'t>, Error<'t>> {
  // match self.peek(
  // }

  fn recipe(&mut self, name: &'a str) -> Result<Recipe<'a>, Error<'a>> {
    let mut arguments = vec![];
    loop {
      if let Some(name_token) = self.accept(Name) {
        if arguments.contains(&name_token.lexeme) {
          return Err(error(self.text, name_token.line, ErrorKind::DuplicateArgument{
            recipe: name, argument: name_token.lexeme}));
        }
        arguments.push(name_token.lexeme);
      } else {
        break;
      }
    }

    self.expect(Colon);

    let mut dependencies = vec![];
    loop {
      if let Some(name_token) = self.accept(Name) {
        if dependencies.contains(&name_token.lexeme) {
          panic!("duplicate dependency");
          // return Err(error(self.text, name_token.line, ErrorKind::DuplicateDependency{
          // name: name_token.lexeme}));
        }
        dependencies.push(name_token.lexeme);
      } else {
        break;
      }
    }

    // if !self.accept_eol() {
    //   return Err(error(self.text, i, ErrorKind::UnparsableDependencies));
    // }

    panic!("we fucked");
    // Ok(Recipe{
    // })
  }
  */

  fn error(self, token: &Token<'a>, kind: ErrorKind<'a>) -> Error<'a> {
    token.error(self.text, kind)
  }

  fn file(mut self) -> Result<Justfile<'a>, Error<'a>> {
    let recipes = BTreeMap::new();

    loop {
      match self.tokens.next() {
        Some(token) => match token.class {
          Eof => break,
          Eol => continue,
          _ => return Err(self.error(&token, ErrorKind::InternalError {
            message: format!("unhandled token class: {:?}", token.class)
          })),
        },
        None => return Err(Error {
          text:   self.text,
          index:  0,
          line:   0,
          column: 0,
          kind:   ErrorKind::InternalError {
            message: "unexpected end of token stream".to_string()
          }
        }),
      }
    }

    /*
    loop {
      if self.accepted(Eof) { break;    }
      if self.accept_eol()  { continue; }

      match self.tokens.next() {
        Some(Token{class: Name, lexeme: name, ..}) => {
          if self.accepted(Equals) {
            panic!("Variable assignment not yet implemented");
          } else {
            if recipes.contains_key(name) {
              // return Err(error(self.text, line, ErrorKind::DuplicateDependency{
              //   name: name,
              // }));
              panic!("duplicate dep");
            }
            let recipe = try!(self.recipe(name));
            recipes.insert(name, recipe);
          }
        }
        _ => panic!("got something else")
      };
    }
    */

    if let Some(ref token) = self.tokens.next() {
      return Err(self.error(token, ErrorKind::InternalError{
        message: format!("unexpected token remaining after parsing completed: {:?}", token.class)
      }))
    }

    Ok(Justfile{recipes: recipes})
  }
}
