extern crate regex;

use std::io::prelude::*;

use std::{io, fs, env, fmt};
use std::collections::{HashSet, BTreeMap};
use std::fmt::Display;
use regex::Regex;

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

struct Recipe<'a> {
  line:               usize,
  name:               &'a str,
  leading_whitespace: &'a str,
  commands:           Vec<&'a str>,
  dependencies:       HashSet<&'a str>,
}

struct Error<'a> {
  text: &'a str,
  line: usize,
  kind: ErrorKind<'a>
}

enum ErrorKind<'a> {
  CircularDependency{circle: Vec<&'a str>},
  DuplicateDependency{name: &'a str},
  DuplicateRecipe{first: usize, name: &'a str},
  InconsistentLeadingWhitespace{expected: &'a str, found: &'a str},
  Shebang,
  UnknownDependency{name: &'a str, unknown: &'a str},
  Unparsable,
  UnparsableDependencies,
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

impl<'a> Display for Error<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try!(write!(f, "justfile:{}: ", self.line));

    match self.kind {
      ErrorKind::CircularDependency{ref circle} => {
        try!(write!(f, "circular dependency: {}", circle.join(" -> ")));
        return Ok(());
      }
      ErrorKind::DuplicateDependency{name} => {
        try!(writeln!(f, "duplicate dependency: {}", name));
      }
      ErrorKind::DuplicateRecipe{first, name} => {
        try!(write!(f, "duplicate recipe: {} appears on lines {} and {}", 
                    name, first, self.line));
        return Ok(());
      }
      ErrorKind::InconsistentLeadingWhitespace{expected, found} => {
        try!(writeln!(f,
          "inconsistant leading whitespace: recipe started with {} but found line with {}:",
          show_whitespace(expected), show_whitespace(found)
        ));
      }
      ErrorKind::Shebang => {
        try!(writeln!(f, "shebang \"#!\" is reserved syntax"))
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
    }

    match self.text.lines().nth(self.line) {
      Some(line) => try!(write!(f, "{}", line)),
      None => die!("internal error: Error has invalid line number: {}", self.line),
    }

    Ok(())
  }
}

struct Justfile<'a> {
  _recipes: BTreeMap<&'a str, Recipe<'a>>
}

fn parse<'a>(text: &'a str) -> Result<Justfile, Error> {
  let shebang_re    = re(r"^\s*#!(.*)$");
  let comment_re    = re(r"^\s*#[^!].*$");
  let command_re    = re(r"^(\s+)(.*)$");
  let blank_re      = re(r"^\s*$");
  let label_re      = re(r"^([a-z](-[a-z]|[a-z])*):(.*)$");
  let name_re       = re(r"^[a-z](-[a-z]|[a-z])*$");
  let whitespace_re = re(r"\s+");

  let mut recipes: BTreeMap<&'a str, Recipe<'a>> = BTreeMap::new();
  let mut current_recipe: Option<Recipe> = None;
  for (i, line) in text.lines().enumerate() {
    if blank_re.is_match(line) {
      continue;
    } else if shebang_re.is_match(line) {
      return Err(error(text, i, ErrorKind::Shebang));
    }

    if let Some(mut recipe) = current_recipe {
      match command_re.captures(line) {
        Some(captures) => {
          let leading_whitespace = captures.at(1).unwrap();
          if recipe.leading_whitespace == "" {
            recipe.leading_whitespace = leading_whitespace;
          } else if !line.starts_with(recipe.leading_whitespace) {
            return Err(error(text, i, ErrorKind::InconsistentLeadingWhitespace{
              expected: recipe.leading_whitespace,
              found:    leading_whitespace,
            }));
          }
          let command = captures.at(2).unwrap();
          recipe.commands.push(command);
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
    } else if let Some(captures) = label_re.captures(line) {
      let name = captures.at(1).unwrap();
      if let Some(recipe) = recipes.get(name) {
        return Err(error(text, i, ErrorKind::DuplicateRecipe {
          first: recipe.line,
          name: name,
        }));
      }

      let rest = captures.at(3).unwrap().trim();
      let mut dependencies = HashSet::new();
      for part in whitespace_re.split(rest) {
        if name_re.is_match(part) {
          if dependencies.contains(part) {
            return Err(error(text, i, ErrorKind::DuplicateDependency{
              name: part,
            }));
          }
          dependencies.insert(part);
        } else {
          return Err(error(text, i, ErrorKind::UnparsableDependencies));
        }
      }

      current_recipe = Some(Recipe{
        line:               i,
        name:               name,
        leading_whitespace: "",
        commands:           vec![],
        dependencies:      dependencies,
      });
    } else {
      return Err(error(text, i, ErrorKind::Unparsable));
    }
  }

  if let Some(recipe) = current_recipe {
    recipes.insert(recipe.name, recipe);
  }

  let mut resolved = HashSet::new();
  let mut seen     = HashSet::new();
  let mut stack    = vec![];

  fn resolve<'a>(
    text:     &'a str,
    recipes:  &BTreeMap<&str, Recipe<'a>>,
    resolved: &mut HashSet<&'a str>,
    seen:     &mut HashSet<&'a str>,
    stack:    &mut Vec<&'a str>,
    recipe:   &Recipe<'a>,
  ) -> Result<(), Error<'a>> {
    stack.push(recipe.name);
    seen.insert(recipe.name);
    for dependency_name in &recipe.dependencies {
      match recipes.get(dependency_name) {
        Some(dependency) => if !resolved.contains(dependency.name) {
          if seen.contains(dependency.name) {
            let first = stack[0];
            stack.push(first);
            return Err(error(text, recipe.line, ErrorKind::CircularDependency {
              circle: stack.iter()
                .skip_while(|name| **name != dependency.name)
                .cloned().collect()
            }));
          }
          return resolve(text, recipes, resolved, seen, stack, dependency);
        },
        None => return Err(error(text, recipe.line, ErrorKind::UnknownDependency {
          name:    recipe.name,
          unknown: dependency_name
        })),
      }
    }
    resolved.insert(recipe.name);
    stack.pop();
    Ok(())
  }

  for (_, ref recipe) in &recipes {
    try!(resolve(text, &recipes, &mut resolved, &mut seen, &mut stack, &recipe));
  }

  Ok(Justfile{_recipes: recipes})
}

fn main() {
  loop {
    match fs::metadata("justfile") {
      Ok(metadata) => if metadata.is_file() { break; },
      Err(error) => {
        if error.kind() != io::ErrorKind::NotFound {
          die!("Error fetching justfile metadata: {}", error)
        }
      }
    }

    match env::current_dir() {
      Ok(pathbuf) => if pathbuf.as_os_str() == "/" { die!("No justfile found."); },
      Err(error) => die!("Error getting current dir: {}", error),
    }

    if let Err(error) = env::set_current_dir("..") {
      die!("Error changing directory: {}", error);
    }
  }

  let text = fs::File::open("justfile")
    .unwrap_or_else(|error| die!("Error opening justfile: {}", error))
    .slurp()
    .unwrap_or_else(|error| die!("Error reading justfile: {}", error));

  let _justfile = parse(&text).unwrap_or_else(|error| die!("{}", error));

  /*
  // let requests: Vec<String> = std::env::args().skip(1).collect();
  // for request in requests {
  //   println!("{}", request);
  // }

  // let arguments: Vec<String> = std::env::args().skip(1 + recipes.len() + 1).collect();

  // for (i, argument) in arguments.into_iter().enumerate() {
    // std::env::set_var(format!("ARG{}", i), argument);
  // }

  let mut command = std::process::Command::new(make.command());

  command.arg("MAKEFLAGS=");

  if make.gnu() {
    command.arg("--always-make").arg("--no-print-directory");
  }

  command.arg("-f").arg("justfile");

  for recipe in recipes {
    command.arg(recipe);
  }

  match command.status() {
    Err(error) => die!("Failed to execute `{:?}`: {}", command, error),
    Ok(exit_status) => match exit_status.code() {
      Some(code) => std::process::exit(code),
      None => std::process::exit(-1),
    }
  }
  */
}
