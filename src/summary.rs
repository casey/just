//! The contents of this module are not bound by any stability guarantees.
//! Breaking changes may be introduced at any time.
//!
//! The main entry point into this module is the `summary` function, which
//! parses a justfile at a given path and produces a `Summary` object,
//! which broadly captures the functionality of the parsed justfile, or
//! an error message.
//!
//! This functionality is intended to be used with `janus`, a tool for
//! ensuring that changes to just do not inadvertantly break or
//! change the interpretation of existing justfiles.

use std::{
  collections::{BTreeMap, BTreeSet},
  fs, io,
  path::Path,
};

use crate::{expression, fragment, justfile::Justfile, parser::Parser, recipe};

pub fn summary(path: impl AsRef<Path>) -> Result<Result<Summary, String>, io::Error> {
  let path = path.as_ref();

  let text = fs::read_to_string(path)?;

  match Parser::parse(&text) {
    Ok(justfile) => Ok(Ok(Summary::new(justfile))),
    Err(compilation_error) => Ok(Err(compilation_error.to_string())),
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Summary {
  pub assignments: BTreeMap<String, Assignment>,
  pub recipes: BTreeMap<String, Recipe>,
}

impl Summary {
  fn new(justfile: Justfile) -> Summary {
    let exports = justfile.exports;

    let mut aliases = BTreeMap::new();

    for alias in justfile.aliases.values() {
      aliases
        .entry(alias.target)
        .or_insert(Vec::new())
        .push(alias.name.to_string());
    }

    Summary {
      recipes: justfile
        .recipes
        .into_iter()
        .map(|(name, recipe)| {
          (
            name.to_string(),
            Recipe::new(recipe, aliases.remove(name).unwrap_or(Vec::new())),
          )
        })
        .collect(),
      assignments: justfile
        .assignments
        .into_iter()
        .map(|(name, expression)| {
          (
            name.to_string(),
            Assignment::new(name, expression, &exports),
          )
        })
        .collect(),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Recipe {
  pub aliases: Vec<String>,
  pub dependencies: BTreeSet<String>,
  pub lines: Vec<Line>,
  pub private: bool,
  pub quiet: bool,
  pub shebang: bool,
}

impl Recipe {
  fn new(recipe: recipe::Recipe, aliases: Vec<String>) -> Recipe {
    Recipe {
      private: recipe.private,
      shebang: recipe.shebang,
      quiet: recipe.quiet,
      dependencies: recipe.dependencies.into_iter().map(str::to_owned).collect(),
      lines: recipe.lines.into_iter().map(Line::new).collect(),
      aliases,
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Line {
  pub fragments: Vec<Fragment>,
}

impl Line {
  fn new(fragments: Vec<fragment::Fragment>) -> Line {
    Line {
      fragments: fragments.into_iter().map(Fragment::new).collect(),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub enum Fragment {
  Text { text: String },
  Expression { expression: Expression },
}

impl Fragment {
  fn new(fragment: fragment::Fragment) -> Fragment {
    match fragment {
      fragment::Fragment::Text { text } => Fragment::Text {
        text: text.lexeme.to_owned(),
      },
      fragment::Fragment::Expression { expression } => Fragment::Expression {
        expression: Expression::new(expression),
      },
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Assignment {
  pub exported: bool,
  pub expression: Expression,
}

impl Assignment {
  fn new(name: &str, expression: expression::Expression, exports: &BTreeSet<&str>) -> Assignment {
    Assignment {
      exported: exports.contains(name),
      expression: Expression::new(expression),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub enum Expression {
  Backtick {
    command: String,
  },
  Call {
    name: String,
    arguments: Vec<Expression>,
  },
  Concatination {
    lhs: Box<Expression>,
    rhs: Box<Expression>,
  },
  String {
    text: String,
  },
  Variable {
    name: String,
  },
}

impl Expression {
  fn new(expression: expression::Expression) -> Expression {
    use expression::Expression::*;
    match expression {
      Backtick { raw, .. } => Expression::Backtick {
        command: raw.to_owned(),
      },
      Call {
        name, arguments, ..
      } => Expression::Call {
        name: name.to_owned(),
        arguments: arguments.into_iter().map(Expression::new).collect(),
      },
      Concatination { lhs, rhs } => Expression::Concatination {
        lhs: Box::new(Expression::new(*lhs)),
        rhs: Box::new(Expression::new(*rhs)),
      },
      String { cooked_string } => Expression::String {
        text: cooked_string.cooked,
      },
      Variable { name, .. } => Expression::Variable {
        name: name.to_owned(),
      },
    }
  }
}
