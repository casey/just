#[cfg(test)]
mod tests;

extern crate regex;

use std::io::prelude::*;

use std::{fs, fmt, process, io};
use std::collections::{BTreeMap, BTreeSet, HashSet};
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
  dependencies:       BTreeSet<&'a str>,
  shebang:            bool,
}

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
    // TODO: if shebang, run as script
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
    }

    match self.text.lines().nth(self.line) {
      Some(line) => try!(write!(f, "{}", line)),
      None => die!("internal error: Error has invalid line number: {}", self.line),
    }

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
    }
    Ok(())
  }
}

pub fn parse<'a>(text: &'a str) -> Result<Justfile, Error> {
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
      let mut dependencies = BTreeSet::new();
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
        line_number:        i,
        label:              line,
        name:               name,
        leading_whitespace: "",
        lines:              vec![],
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
