extern crate regex;

use std::io::prelude::*;

use std::{io, fs, env};
use std::collections::{HashSet, BTreeMap};

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

fn re(pattern: &str) -> regex::Regex {
  regex::Regex::new(pattern).unwrap()
}

struct Recipe<'a> {
  _line:               u64,
  name:               &'a str,
  leading_whitespace: &'a str,
  commands:           Vec<&'a str>,
  dependencies:       HashSet<&'a str>,
}

struct Resolver<'a> {
  recipes:  &'a BTreeMap<&'a str, Recipe<'a>>,
  resolved: HashSet<&'a str>,
  seen:     HashSet<&'a str>,
  stack:    Vec<&'a str>,
}

fn resolve<'a> (recipes: &'a BTreeMap<&'a str, Recipe<'a>>) {
  let mut resolver = Resolver {
    recipes:  recipes,
    resolved: HashSet::new(),
    seen:     HashSet::new(),
    stack:    vec![],
  };

  for (_, recipe) in recipes {
    resolver.resolve(recipe);
  }
}

impl<'a> Resolver<'a> {
  fn resolve(&mut self, recipe: &'a Recipe) {
    self.stack.push(recipe.name);
    self.seen.insert(recipe.name);
    for dependency_name in &recipe.dependencies {
      match self.recipes.get(dependency_name) {
        Some(dependency) => if !self.resolved.contains(dependency.name) {
          if self.seen.contains(dependency.name) {
            let first = self.stack[0];
            self.stack.push(first);
            die!("Circular dependency: {}",
              self.stack.iter()
                .skip_while(|name| **name != dependency.name)
                .cloned().collect::<Vec<&str>>().join(" -> "));
          }
          self.resolve(dependency);
        },
        None => die!("Recipe \"{}\" depends on recipe \"{}\", which doesn't exist.",
          recipe.name, dependency_name),
      }

    }
    self.resolved.insert(recipe.name);
    self.stack.pop();
  }
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

  let mut contents = String::new();

  fs::File::open("justfile")
    .unwrap_or_else(|error| die!("Error opening justfile: {}", error))
    .read_to_string(&mut contents)
    .unwrap_or_else(|error| die!("Error reading justfile: {}", error));

  let shebang_re    = re(r"^\s*#!(.*)$");
  let comment_re    = re(r"^\s*#[^!].*$");
  let command_re    = re(r"^(\s+)(.*)$");
  let blank_re      = re(r"^\s*$");
  let label_re      = re(r"^([a-z](-[a-z]|[a-z])*):(.*)$");
  let name_re       = re(r"^[a-z](-[a-z]|[a-z])*$");
  let whitespace_re = re(r"\s+");

  let mut recipes = BTreeMap::new();
  let mut current_recipe: Option<Recipe> = None;
  for (i, line) in contents.lines().enumerate() {
    if blank_re.is_match(line) {
      continue;
    } else if shebang_re.is_match(line) {
      die!("Unexpected shebang on line {}: {}", i, line);
    }

    if let Some(mut recipe) = current_recipe {
      match command_re.captures(line) {
        Some(captures) => {
          let leading_whitespace = captures.at(1).unwrap();
          if recipe.leading_whitespace == "" {
            recipe.leading_whitespace = leading_whitespace;
          } else if leading_whitespace != recipe.leading_whitespace {
            die!("Command on line {} has inconsistent leading whitespace: {}",
              i, line);
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
      let rest = captures.at(3).unwrap().trim();
      let mut dependencies = HashSet::new();
      for part in whitespace_re.split(rest) {
        if name_re.is_match(part) {
          if dependencies.contains(part) {
            die!("Duplicate dependency \"{}\" on line {}", part, i);
          }
          dependencies.insert(part);
        } else {
          die!("Bad label on line {}: {}", i, line);
        }
      }
      
      if recipes.contains_key(name) {
        die!("Duplicate recipe name \"{}\" on line {}.", name, i);
      }

      current_recipe = Some(Recipe{
        _line:               i as u64,
        name:               name,
        leading_whitespace: "",
        commands:           vec![],
        dependencies:      dependencies,
      });
    } else {
      die!("Error parsing line {} of justfile: {}", i, line);
    }
  }

  if let Some(recipe) = current_recipe {
    recipes.insert(recipe.name, recipe);
  }

  resolve(&recipes);

  // let requests: Vec<String> = std::env::args().skip(1).collect();
  // for request in requests {
  //   println!("{}", request);
  // }

  // let arguments: Vec<String> = std::env::args().skip(1 + recipes.len() + 1).collect();

  // for (i, argument) in arguments.into_iter().enumerate() {
    // std::env::set_var(format!("ARG{}", i), argument);
  // }

  /*
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
