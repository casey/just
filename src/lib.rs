#[cfg(test)]
mod tests;

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
  leading_whitespace: &'a str,
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

#[derive(Debug)]
pub struct Error<'a> {
  text: &'a str,
  line: usize,
  kind: ErrorKind<'a>
}

#[derive(Debug, PartialEq)]
enum ErrorKind<'a> {
  BadRecipeName{name: &'a str},
  CircularDependency{circle: Vec<&'a str>},
  DuplicateDependency{name: &'a str},
  DuplicateArgument{recipe: &'a str, argument: &'a str},
  DuplicateRecipe{first: usize, name: &'a str},
  TabAfterSpace{whitespace: &'a str},
  MixedLeadingWhitespace{whitespace: &'a str},
  ExtraLeadingWhitespace,
  InconsistentLeadingWhitespace{expected: &'a str, found: &'a str},
  OuterShebang,
  NonLeadingShebang{recipe: &'a str},
  UnknownDependency{name: &'a str, unknown: &'a str},
  Unparsable,
  UnparsableDependencies,
  UnknownStartOfToken,
}

fn error<'a>(text: &'a str, line: usize, kind: ErrorKind<'a>) 
  -> Error<'a>
{
  Error {
    text: text,
    line: line,
    kind: kind,
  }
}

fn show_whitespace(text: &str) -> String {
  text.chars().map(|c| match c { '\t' => 't', ' ' => 's', _ => c }).collect()
}

fn mixed(text: &str) -> bool {
  !(text.chars().all(|c| c == ' ') || text.chars().all(|c| c == '\t'))
}

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

impl<'a> Display for Error<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "justfile:{}: ", self.line));

    match self.kind {
      ErrorKind::BadRecipeName{name} => {
        try!(writeln!(f, "recipe name does not match /[a-z](-[a-z]|[a-z])*/: {}", name));
      }
      ErrorKind::CircularDependency{ref circle} => {
        try!(write!(f, "circular dependency: {}", circle.join(" -> ")));
        return Ok(());
      }
      ErrorKind::DuplicateArgument{recipe, argument} => {
        try!(writeln!(f, "recipe {} has duplicate argument: {}", recipe, argument));
      }
      ErrorKind::DuplicateDependency{name} => {
        try!(writeln!(f, "duplicate dependency: {}", name));
      }
      ErrorKind::DuplicateRecipe{first, name} => {
        try!(write!(f, "duplicate recipe: {} appears on lines {} and {}", 
                    name, first, self.line));
        return Ok(());
      }
      ErrorKind::TabAfterSpace{whitespace} => {
        try!(writeln!(f, "found tab after space: {}", show_whitespace(whitespace)));
      }
      ErrorKind::MixedLeadingWhitespace{whitespace} => {
        try!(writeln!(f,
          "inconsistant leading whitespace: recipe started with {}:",
          show_whitespace(whitespace)
        ));
      }
      ErrorKind::ExtraLeadingWhitespace => {
        try!(writeln!(f, "line has extra leading whitespace"));
      }
      ErrorKind::InconsistentLeadingWhitespace{expected, found} => {
        try!(writeln!(f,
          "inconsistant leading whitespace: recipe started with {} but found line with {}:",
          show_whitespace(expected), show_whitespace(found)
        ));
      }
      ErrorKind::OuterShebang => {
        try!(writeln!(f, "a shebang \"#!\" is reserved syntax outside of recipes"))
      }
      ErrorKind::NonLeadingShebang{..} => {
        try!(writeln!(f, "a shebang \"#!\" may only appear on the first line of a recipe"))
      }
      ErrorKind::UnknownDependency{name, unknown} => {
        try!(writeln!(f, "recipe {} has unknown dependency {}", name, unknown));
      }
      ErrorKind::Unparsable => {
        try!(writeln!(f, "could not parse line:"));
      }
      ErrorKind::UnparsableDependencies => {
        try!(writeln!(f, "could not parse dependencies:"));
      }
      ErrorKind::UnknownStartOfToken => {
        try!(writeln!(f, "uknown start of token:"));
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
  // index:  usize,
  line:   usize,
  // col:    usize,
  prefix: &'a str,
  lexeme: &'a str,
  class:  TokenClass,
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
  let name_re    = token(r"[a-z]((_|-)?[a-z0-9])*");
  let colon_re   = token(r":"                     );
  let equals_re  = token(r"="                     );
  let comment_re = token(r"#([^!].*)?$"           );
  //let shebang_re = token(r"#!"                    );
  let eol_re     = token(r"\n|\r\n"               );
  let eof_re     = token(r"(?-m)$"                );
  //let line_re    = token(r"[^\n\r]"                );

  //let split_re  = re("(?m)$");
  //let body_re   = re(r"^(?ms)(.*?$)\s*(^[^ \t\r\n]|(?-m:$))");
  // let dedent_re = re(r"^(?m)\s*(^[^\s]|(?-m:$))");

  let line_re = re(r"^(?m)[ \t]+[^ \t\n\r].*$");

  /*
  #[derive(PartialEq)]
  enum State<'a> {
    Normal, // starting state
    Colon,  // we have seen a colon since the last eol
    Recipe, // we are on the line after a colon
    Body{indent: &'a str},   // we are in a recipe body
  }
  */

  // state is:
  //   beginning of line or not
  //   current indent

  fn indentation(text: &str) -> Option<&str> {
    // fix this so it isn't recompiled every time
    let indent_re = re(r"^([ \t]*)[^ \t\n\r]");
    indent_re.captures(text).map(|captures| captures.at(1).unwrap())
  }

  let mut tokens     = vec![];
  let mut rest       = text;
  // let mut index      = 0;
  let mut line       = 0;
  let mut col        = 0;
  let mut indent: Option<&str> = None;
  // let mut line   = 0;
  // let mut col    = 0;
  // let mut state  = State::Normal;
  // let mut line_start = true;
  loop {
    if col == 0 {
      if let Some(class) = match (indent, indentation(rest)) {
        // dedent
        (Some(_), Some("")) => {
          indent = None;
          Some(Dedent)
        }
        (None, Some("")) => {
          None
        }
        // indent
        (None, Some(current @ _)) => {
          // check mixed leading whitespace
          indent = Some(current);
          Some(Indent)
        }
        (Some(previous), Some(current @ _)) => {
          if !current.starts_with(previous) {
            return Err(error(text, line, 
              ErrorKind::InconsistentLeadingWhitespace{expected: previous, found: current}
            ));
          }
          None
          // check tabs after spaces
        }
        // ignore
        _ => {
          None
        }
      } {
        tokens.push(Token {
          // index:  index,
          line:   line,
          // col:    col,
          prefix: "",
          lexeme: "",
          class:  class,
        });
      }
    }

    let (prefix, lexeme, class) = 
    if let (0, Some(indent), Some(captures)) = (col, indent, line_re.captures(rest)) {
      let line = captures.at(0).unwrap();
      if !line.starts_with(indent) {
        panic!("Line did not start with expected indentation");
      }
      let (prefix, lexeme) = line.split_at(indent.len());
      (prefix, lexeme, Line)
    } else if let Some(captures) = name_re.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Name)
    } else if let Some(captures) = eol_re.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Eol)
    } else if let Some(captures) = eof_re.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Eof)
    } else if let Some(captures) = colon_re.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Colon)
    } else if let Some(captures) = equals_re.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Equals)
    } else if let Some(captures) = comment_re.captures(rest) {
      (captures.at(1).unwrap(), captures.at(2).unwrap(), Comment)
    } else {
      return Err(if rest.starts_with("#!") {
        error(text, line, ErrorKind::OuterShebang)
      } else {
        error(text, line, ErrorKind::UnknownStartOfToken)
      });
    };
    

    // let (captures, class) = if let (0, Some(captures)) = line_re.captures(rest) {

    /*
    */

    /*
    if state == State::Recipe {
      let captures = indent_re.captures(rest).unwrap();
      let indent = captures.at(1).unwrap();
      let text = captures.at(2).unwrap();
      if indent != "" && text != "" {
        tokens.push(Token {
          index:  index,
          prefix: "",
          lexeme: "",
          class:  TokenClass::Indent,
        });
        state = State::Body{indent: indent};
      } else {
        state = State::Normal;
      }
    }
    */
      /*
      State::Body{indent: _} => {
        if let Some(captures) = body_re.captures(rest) {
          let body_text = captures.at(1).unwrap();
          for mut line in split_re.split(body_text) {
            if let Some(captures) = line_re.captures(line) {
              let len = captures.at(0).unwrap().len();
              tokens.push(Token {
                index:  index,
                prefix: captures.at(1).unwrap(),
                lexeme: captures.at(2).unwrap(),
                class:  TokenClass::Eol,
              });
              line = &line[len..];
            }
            println!("{:?}", line);
          }

          panic!("matched body: {}", captures.at(1).unwrap());


          // split the body into lines
          // for each line in the body, push a line if nonblank, then an eol
          // push a dedent
        }
      },
      */
      // State::Normal | State::Colon | State::Body{..} => {
    /*
    let (captures, class) = if let Some(captures) = eol_re.captures(rest) {
      (captures, TokenClass::Eol)
    } else if let State::Body{indent} = state {
      if dedent_re.is_match(rest) {
        tokens.push(Token {
          index:  index,
          prefix: "",
          lexeme: "",
          class:  TokenClass::Dedent,
        });
        state = State::Normal;
        continue
      }

      if let Some(captures) = line_re.captures(rest) {
        (captures, TokenClass::Line)
      } else {
        panic!("Failed to match a line");
      }
    } else if let Some(captures) = anchor_re.captures(rest) {
      (captures, TokenClass::Anchor)
    } else if let Some(captures) = name_re.captures(rest) {
      (captures, TokenClass::Name)
    } else if let Some(captures) = colon_re.captures(rest) {
      (captures, TokenClass::Colon)
    } else if let Some(captures) = comment_re.captures(rest) {
      let text = captures.at(3).unwrap_or("");
      (captures, TokenClass::Comment{text: text})
    } else if let Some(captures) = eof_re.captures(rest) {
      (captures, TokenClass::Eof)
    } else {
      panic!("Did not match a token! Rest: {}", rest);
    };
    */

    // let (captures, class) = if let (true, Some(captures)) = (line_start, 

    // let all    = captures.at(0).unwrap();
    // let prefix = captures.at(1).unwrap();
    // let lexeme = captures.at(2).unwrap();
    // let len    = all.len();
    // let eof    = class == TokenClass::Eof;
    //assert!(eof || lexeme.len() > 0);
    //assert!(all.len() > 0);
    //assert!(prefix.len() + lexeme.len() == len);

    /*
    if class == TokenClass::Colon {
      state = State::Colon;
    } else if class == TokenClass::Eol && state == State::Colon {
      state = State::Recipe;
    }
    */


    /*
    if class == TokenClass::Eol {
      row += 1;
      col = 0;
    } else {
      col += len;
    }

    let eof = TokenClass::Eof {
    }
    */

    let len = prefix.len() + lexeme.len();

    tokens.push(Token {
      // index:  index,
      line:    line,
      // col:    col,
      prefix: prefix,
      lexeme: lexeme,
      class:  class,
    });

    match tokens.last().unwrap().class {
      Eol => {
        line += 1;
        col = 0;
      },
      Eof => {
        break;
      },
      _ => {
        col += len;
      }
    }

    rest = &rest[len..];
    // index += len;
  }

  Ok(tokens)
}

/*
struct Parser<'a, I> {
  tokens: Vec<Token<'a>>,
  index:  usize,
}
*/

//impl<'a> Parser<'a> {
  /*
  fn peek(&mut self) -> TokenClass {
    self.tokens[self.index].class
  }

  fn advance(&mut self) {
    self.index += 1;
  }

  fn accept_eol(&mut self) -> bool {
    if self.accept(TokenClass::Comment) {
      self.expect(TokenClass::Eol);
      true
    } else
  }
  */

  /*
  fn accept(&mut self, class: TokenClass) -> bool {
    if self.tokens[self.index].class == class {
      self.index += 1;
      true
    } else {
      false
    }
  }
  */

  /*
  fn peek(&mut self) -> Option<TokenClass> {
    self.tokens.get(self.index).map(|t| t.class)
  }

  fn file(mut self) -> Result<Justfile<'a>, Error<'a>> {
    let recipes = BTreeMap::new();

    loop {
      let ref current = self.tokens[self.index];
      self.index += 1;
      
      match current.class {
        TokenClass::Eof     => break,
        TokenClass::Comment => continue,
        TokenClass::Eol     => continue,
        TokenClass::Name    => {
          match self.peek() {
            Some(TokenClass::Name) | Some(TokenClass::Colon) => {
              panic!("time to parse a recipe");
            }
            Some(TokenClass::Equals) => {
              panic!("time to parse an assignment");
            }
            Some(unexpected @ _) => {
              panic!("unexpected token");
            }
            None => {
              panic!("unexpected end of token stream");
            }
          }
        }
        unexpected @ _ => {
          panic!("unexpected token at top level");
        }
      }
    }

    Ok(Justfile{recipes: recipes})
  }
}
*/

// struct Parser<'a, I> where I: std::iter::Iterator<Item=Token<'a>> {
//   tokens: std::iter::Peekable<I>,
// }

struct Parser<'i, 't: 'i> {
  text:   &'t str,
  tokens: &'i mut std::iter::Peekable<std::slice::Iter<'i, Token<'t>>>
}

impl<'i, 't> Parser<'i, 't> {
  fn accept(&mut self, class: TokenClass) -> Option<&Token<'t>> {
    if self.tokens.peek().unwrap().class == class {
      Some(self.tokens.next().unwrap())
    } else {
      None
    }
  }

  fn accepted(&mut self, class: TokenClass) -> bool {
    self.accept(class).is_some()
  }

  fn expect(&mut self, class: TokenClass) {
    if !self.accepted(class) {
      panic!("we fucked");
    }
  }

  fn peek(&mut self, class: TokenClass) -> bool {
    self.tokens.peek().unwrap().class == class
  }

  fn accept_eol(&mut self) -> bool {
    if self.accepted(Comment) {
      if !self.peek(Eof) { self.expect(Eol) };
      true
    } else {
      self.accepted(Eol)
    }
  }

  // fn accept(&mut self) -> Result<Token<'t>, Error<'t>> {
  // match self.peek(
  // }

  fn recipe(&mut self, name: &'t str) -> Result<Recipe<'t>, Error<'t>> {
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
          return Err(error(self.text, name_token.line, ErrorKind::DuplicateDependency{
            name: name_token.lexeme}));
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

  fn file(mut self) -> Result<Justfile<'t>, Error<'t>> {
    let mut recipes = BTreeMap::new();

    loop {
      if self.accepted(Eof) { break;    }
      if self.accept_eol()  { continue; }

      match self.tokens.next() {
        Some(&Token{class: Name, line, lexeme: name, ..}) => {
          if self.accepted(Equals) {
            panic!("Variable assignment not yet implemented");
          } else {
            if recipes.contains_key(name) {
              return Err(error(self.text, line, ErrorKind::DuplicateDependency{
                name: name,
              }));
            }
            let recipe = try!(self.recipe(name));
            recipes.insert(name, recipe);
          }
        }
        _ => panic!("got something else")
      };
    }

    // assert that token.next() == None

    Ok(Justfile{recipes: recipes})
  }
}


// impl<'a, I> Parser<'a, I> where I: std::iter::Iterator<Item=Token<'a>> {
//   fn file(mut self) -> Result<Justfile<'a>, Error<'a>> {
//     Ok()
//   }
// }

pub fn parse<'a>(text: &'a str) -> Result<Justfile, Error> {
  let tokens = try!(tokenize(text));
  // let parser = Parser{tokens: tokens, index: 0};
  // try!(parser.file());

  let parser = Parser{text: text, tokens: &mut tokens.iter().peekable()};
  try!(parser.file());

  let shebang_re    = re(r"^\s*#!(.*)$"           );
  let comment_re    = re(r"^\s*#([^!].*)?$"       );
  let command_re    = re(r"^(\s+).*$"             );
  let blank_re      = re(r"^\s*$"                 );
  let label_re      = re(r"^([^#]*):(.*)$"        );
  let name_re       = re(r"^[a-z](-[a-z]|[a-z])*$");
  let whitespace_re = re(r"\s+"                   );

  let mut recipes: BTreeMap<&'a str, Recipe<'a>> = BTreeMap::new();
  let mut current_recipe: Option<Recipe> = None;
  for (i, line) in text.lines().enumerate() {
    if blank_re.is_match(line) {
      continue;
    }

    if let Some(mut recipe) = current_recipe {
      match command_re.captures(line) {
        Some(captures) => {
          let leading_whitespace = captures.at(1).unwrap();
          if tab_after_space(leading_whitespace) {
            return Err(error(text, i, ErrorKind::TabAfterSpace{
              whitespace: leading_whitespace,
            }));
          } else if recipe.leading_whitespace == "" {
            if mixed(leading_whitespace) {
              return Err(error(text, i, ErrorKind::MixedLeadingWhitespace{
                whitespace: leading_whitespace
              }));
            }
            recipe.leading_whitespace = leading_whitespace;
          } else if !line.starts_with(recipe.leading_whitespace) {
            return Err(error(text, i, ErrorKind::InconsistentLeadingWhitespace{
              expected: recipe.leading_whitespace,
              found:    leading_whitespace,
            }));
          }
          recipe.lines.push(line.split_at(recipe.leading_whitespace.len()).1);
          current_recipe = Some(recipe);
          continue;
        },
        None => {
          recipes.insert(recipe.name, recipe);
          current_recipe = None;
        },
      }
    }
    
    if comment_re.is_match(line) {
      // ignore
    } else if shebang_re.is_match(line) {
      return Err(error(text, i, ErrorKind::OuterShebang));
    } else if let Some(captures) = label_re.captures(line) {
      let name = captures.at(1).unwrap();
      if !name_re.is_match(name) {
        return Err(error(text, i, ErrorKind::BadRecipeName {
          name: name,
        }));
      }
      if let Some(recipe) = recipes.get(name) {
        return Err(error(text, i, ErrorKind::DuplicateRecipe {
          first: recipe.line_number,
          name: name,
        }));
      }

      let rest = captures.at(2).unwrap().trim();
      let mut dependencies = vec![];
      for part in whitespace_re.split(rest) {
        if name_re.is_match(part) {
          if dependencies.contains(&part) {
            return Err(error(text, i, ErrorKind::DuplicateDependency{
              name: part,
            }));
          }
          dependencies.push(part);
        } else {
          return Err(error(text, i, ErrorKind::UnparsableDependencies));
        }
      }

      current_recipe = Some(Recipe{
        line_number:        i,
        label:              line,
        name:               name,
        leading_whitespace: "",
        lines:              vec![],
        // fragments:          vec![],
        // variables:          BTreeSet::new(),
        // arguments:          vec![],
        dependencies:       dependencies,
        shebang:            false,
      });
    } else {
      return Err(error(text, i, ErrorKind::Unparsable));
    }
  }

  if let Some(recipe) = current_recipe {
    recipes.insert(recipe.name, recipe);
  }

  let leading_whitespace_re = re(r"^\s+");

  for recipe in recipes.values_mut() {
    for (i, line) in recipe.lines.iter().enumerate() {
      let line_number = recipe.line_number + 1 + i;
      if shebang_re.is_match(line) {
        if i == 0 {
          recipe.shebang = true;
        } else {
          return Err(error(text, line_number, ErrorKind::NonLeadingShebang{recipe: recipe.name}));
        }
      }
      if !recipe.shebang && leading_whitespace_re.is_match(line) {
        return Err(error(text, line_number, ErrorKind::ExtraLeadingWhitespace));
      }
    }
  }

  let mut resolved = HashSet::new();
  let mut seen     = HashSet::new();
  let mut stack    = vec![];

  for (_, ref recipe) in &recipes {
    try!(resolve(text, &recipes, &mut resolved, &mut seen, &mut stack, &recipe));
  }

  Ok(Justfile{recipes: recipes})
}
