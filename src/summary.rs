//! Justfile summary creation, for testing purposes only.
//!
//! The contents of this module are not bound by any stability guarantees.
//! Breaking changes may be introduced at any time.
//!
//! The main entry point into this module is the `summary` function, which
//! parses a justfile at a given path and produces a `Summary` object,
//! which broadly captures the functionality of the parsed justfile, or
//! an error message.
//!
//! This functionality is intended to be used with `janus`, a tool for
//! ensuring that changes to just do not inadvertently break or
//! change the interpretation of existing justfiles.

use std::{
  collections::{BTreeMap, BTreeSet},
  fs, io,
  path::Path,
};

use crate::compiler::Compiler;

mod full {
  pub(crate) use crate::{
    assignment::Assignment, expression::Expression, fragment::Fragment, justfile::Justfile,
    line::Line, parameter::Parameter, recipe::Recipe,
  };
}

pub fn summary(path: &Path) -> Result<Result<Summary, String>, io::Error> {
  let text = fs::read_to_string(path)?;

  match Compiler::compile(&text) {
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
  fn new(justfile: full::Justfile) -> Summary {
    let mut aliases = BTreeMap::new();

    for alias in justfile.aliases.values() {
      aliases
        .entry(alias.target.lexeme())
        .or_insert_with(Vec::new)
        .push(alias.name.to_string());
    }

    Summary {
      recipes: justfile
        .recipes
        .into_iter()
        .map(|(name, recipe)| {
          (
            name.to_string(),
            Recipe::new(recipe, aliases.remove(name).unwrap_or_default()),
          )
        })
        .collect(),
      assignments: justfile
        .assignments
        .into_iter()
        .map(|(name, assignment)| (name.to_string(), Assignment::new(assignment)))
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
  pub parameters: Vec<Parameter>,
}

impl Recipe {
  fn new(recipe: full::Recipe, aliases: Vec<String>) -> Recipe {
    Recipe {
      private: recipe.private,
      shebang: recipe.shebang,
      quiet: recipe.quiet,
      dependencies: recipe
        .dependencies
        .into_iter()
        .map(|name| name.lexeme().to_string())
        .collect(),
      lines: recipe.body.into_iter().map(Line::new).collect(),
      parameters: recipe.parameters.into_iter().map(Parameter::new).collect(),
      aliases,
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Parameter {
  pub variadic: bool,
  pub name: String,
  pub default: Option<Expression>,
}

impl Parameter {
  fn new(parameter: full::Parameter) -> Parameter {
    Parameter {
      variadic: parameter.variadic,
      name: parameter.name.lexeme().to_owned(),
      default: parameter.default.map(Expression::new),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Line {
  pub fragments: Vec<Fragment>,
}

impl Line {
  fn new(line: full::Line) -> Line {
    Line {
      fragments: line.fragments.into_iter().map(Fragment::new).collect(),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub enum Fragment {
  Text { text: String },
  Expression { expression: Expression },
}

impl Fragment {
  fn new(fragment: full::Fragment) -> Fragment {
    match fragment {
      full::Fragment::Text { token } => Fragment::Text {
        text: token.lexeme().to_owned(),
      },
      full::Fragment::Interpolation { expression } => Fragment::Expression {
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
  fn new(assignment: full::Assignment) -> Assignment {
    Assignment {
      exported: assignment.export,
      expression: Expression::new(assignment.expression),
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
  fn new(expression: full::Expression) -> Expression {
    use full::Expression::*;
    match expression {
      Backtick { contents, .. } => Expression::Backtick {
        command: contents.to_owned(),
      },
      Call {
        function,
        arguments,
      } => Expression::Call {
        name: function.lexeme().to_owned(),
        arguments: arguments.into_iter().map(Expression::new).collect(),
      },
      Concatination { lhs, rhs } => Expression::Concatination {
        lhs: Box::new(Expression::new(*lhs)),
        rhs: Box::new(Expression::new(*rhs)),
      },
      StringLiteral { string_literal } => Expression::String {
        text: string_literal.cooked.to_string(),
      },
      Variable { name, .. } => Expression::Variable {
        name: name.lexeme().to_owned(),
      },
      Group { contents } => Expression::new(*contents),
    }
  }
}
