//! Justfile summary creation, for testing purposes only.
//!
//! The contents of this module are not bound by any stability guarantees.
//! Breaking changes may be introduced at any time.
//!
//! The main entry point into this module is the `summary` function, which
//! parses a justfile at a given path and produces a `Summary` object, which
//! broadly captures the functionality of the parsed justfile, or an error
//! message.
//!
//! This functionality is intended to be used with `janus`, a tool for ensuring
//! that changes to just do not inadvertently break or change the interpretation
//! of existing justfiles.

use std::{collections::BTreeMap, fs, io, path::Path};

use crate::compiler::Compiler;

mod full {
  pub(crate) use crate::{
    assignment::Assignment, conditional_operator::ConditionalOperator, dependency::Dependency,
    expression::Expression, fragment::Fragment, justfile::Justfile, line::Line,
    parameter::Parameter, parameter_kind::ParameterKind, recipe::Recipe, thunk::Thunk,
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
        .entry(alias.target.name())
        .or_insert_with(Vec::new)
        .push(alias.name.to_string());
    }

    Summary {
      recipes: justfile
        .recipes
        .into_iter()
        .map(|(name, recipe)| {
          (
            name.to_owned(),
            Recipe::new(&recipe, aliases.remove(name).unwrap_or_default()),
          )
        })
        .collect(),
      assignments: justfile
        .assignments
        .iter()
        .map(|(name, assignment)| ((*name).to_owned(), Assignment::new(assignment)))
        .collect(),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Recipe {
  pub aliases: Vec<String>,
  pub dependencies: Vec<Dependency>,
  pub lines: Vec<Line>,
  pub private: bool,
  pub quiet: bool,
  pub shebang: bool,
  pub parameters: Vec<Parameter>,
}

impl Recipe {
  fn new(recipe: &full::Recipe, aliases: Vec<String>) -> Recipe {
    Recipe {
      private: recipe.private,
      shebang: recipe.shebang,
      quiet: recipe.quiet,
      dependencies: recipe.dependencies.iter().map(Dependency::new).collect(),
      lines: recipe.body.iter().map(Line::new).collect(),
      parameters: recipe.parameters.iter().map(Parameter::new).collect(),
      aliases,
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Parameter {
  pub kind: ParameterKind,
  pub name: String,
  pub default: Option<Expression>,
}

impl Parameter {
  fn new(parameter: &full::Parameter) -> Parameter {
    Parameter {
      kind: ParameterKind::new(parameter.kind),
      name: parameter.name.lexeme().to_owned(),
      default: parameter.default.as_ref().map(Expression::new),
    }
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ParameterKind {
  Singular,
  Plus,
  Star,
}

impl ParameterKind {
  fn new(parameter_kind: full::ParameterKind) -> Self {
    match parameter_kind {
      full::ParameterKind::Singular => Self::Singular,
      full::ParameterKind::Plus => Self::Plus,
      full::ParameterKind::Star => Self::Star,
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Line {
  pub fragments: Vec<Fragment>,
}

impl Line {
  fn new(line: &full::Line) -> Line {
    Line {
      fragments: line.fragments.iter().map(Fragment::new).collect(),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub enum Fragment {
  Text { text: String },
  Expression { expression: Expression },
}

impl Fragment {
  fn new(fragment: &full::Fragment) -> Fragment {
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
  fn new(assignment: &full::Assignment) -> Assignment {
    Assignment {
      exported: assignment.export,
      expression: Expression::new(&assignment.value),
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
  Concatenation {
    lhs: Box<Expression>,
    rhs: Box<Expression>,
  },
  Conditional {
    lhs: Box<Expression>,
    rhs: Box<Expression>,
    then: Box<Expression>,
    otherwise: Box<Expression>,
    operator: ConditionalOperator,
  },
  Join {
    lhs: Option<Box<Expression>>,
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
  fn new(expression: &full::Expression) -> Expression {
    use full::Expression::*;
    match expression {
      Backtick { contents, .. } => Expression::Backtick {
        command: (*contents).clone(),
      },
      Call { thunk } => match thunk {
        full::Thunk::Nullary { name, .. } => Expression::Call {
          name: name.lexeme().to_owned(),
          arguments: Vec::new(),
        },
        full::Thunk::Unary { name, arg, .. } => Expression::Call {
          name: name.lexeme().to_owned(),
          arguments: vec![Expression::new(arg)],
        },
        full::Thunk::Binary {
          name, args: [a, b], ..
        } => Expression::Call {
          name: name.lexeme().to_owned(),
          arguments: vec![Expression::new(a), Expression::new(b)],
        },
        full::Thunk::BinaryPlus {
          name,
          args: ([a, b], rest),
          ..
        } => {
          let mut arguments = vec![Expression::new(a), Expression::new(b)];
          for arg in rest {
            arguments.push(Expression::new(arg));
          }
          Expression::Call {
            name: name.lexeme().to_owned(),
            arguments,
          }
        }
        full::Thunk::Ternary {
          name,
          args: [a, b, c],
          ..
        } => Expression::Call {
          name: name.lexeme().to_owned(),
          arguments: vec![Expression::new(a), Expression::new(b), Expression::new(c)],
        },
      },
      Concatenation { lhs, rhs } => Expression::Concatenation {
        lhs: Box::new(Expression::new(lhs)),
        rhs: Box::new(Expression::new(rhs)),
      },
      Join { lhs, rhs } => Expression::Join {
        lhs: lhs.as_ref().map(|lhs| Box::new(Expression::new(lhs))),
        rhs: Box::new(Expression::new(rhs)),
      },
      Conditional {
        lhs,
        operator,
        otherwise,
        rhs,
        then,
      } => Expression::Conditional {
        lhs: Box::new(Expression::new(lhs)),
        operator: ConditionalOperator::new(*operator),
        otherwise: Box::new(Expression::new(otherwise)),
        rhs: Box::new(Expression::new(rhs)),
        then: Box::new(Expression::new(then)),
      },
      StringLiteral { string_literal } => Expression::String {
        text: string_literal.cooked.clone(),
      },
      Variable { name, .. } => Expression::Variable {
        name: name.lexeme().to_owned(),
      },
      Group { contents } => Expression::new(contents),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub enum ConditionalOperator {
  Equality,
  Inequality,
  RegexMatch,
}

impl ConditionalOperator {
  fn new(operator: full::ConditionalOperator) -> Self {
    match operator {
      full::ConditionalOperator::Equality => Self::Equality,
      full::ConditionalOperator::Inequality => Self::Inequality,
      full::ConditionalOperator::RegexMatch => Self::RegexMatch,
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Dependency {
  pub recipe: String,
  pub arguments: Vec<Expression>,
}

impl Dependency {
  fn new(dependency: &full::Dependency) -> Self {
    Self {
      recipe: dependency.recipe.name().to_owned(),
      arguments: dependency.arguments.iter().map(Expression::new).collect(),
    }
  }
}
