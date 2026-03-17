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

use {
  crate::{compiler::Compiler, config::Config, error::Error, loader::Loader},
  std::{collections::BTreeMap, io, path::Path},
};

mod full {
  pub(crate) use crate::{
    assignment::Assignment, condition::Condition, conditional_operator::ConditionalOperator,
    dependency::Dependency, expression::Expression, fragment::Fragment, justfile::Justfile,
    line::Line, parameter::Parameter, parameter_kind::ParameterKind, recipe::Recipe, thunk::Thunk,
  };
}

pub fn summary(path: &Path) -> io::Result<Result<Summary, String>> {
  let loader = Loader::new();

  match Compiler::compile(&Config::default(), &loader, path) {
    Ok(compilation) => Ok(Ok(Summary::new(&compilation.justfile))),
    Err(error) => Ok(Err(if let Error::Compile { compile_error } = error {
      compile_error.to_string()
    } else {
      format!("{error:?}")
    })),
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Summary {
  pub assignments: BTreeMap<String, Assignment>,
  pub recipes: BTreeMap<String, Recipe>,
}

impl Summary {
  fn new(justfile: &full::Justfile) -> Self {
    let mut aliases = BTreeMap::new();

    for alias in justfile.aliases.values() {
      aliases
        .entry(alias.target.name())
        .or_insert_with(Vec::new)
        .push(alias.name.to_string());
    }

    Self {
      recipes: justfile
        .recipes
        .iter()
        .map(|(name, recipe)| {
          (
            (*name).to_string(),
            Recipe::new(recipe, aliases.remove(name).unwrap_or_default()),
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
  pub parameters: Vec<Parameter>,
  pub private: bool,
  pub quiet: bool,
  pub shebang: bool,
}

impl Recipe {
  fn new(recipe: &full::Recipe, aliases: Vec<String>) -> Self {
    Self {
      aliases,
      dependencies: recipe.dependencies.iter().map(Dependency::new).collect(),
      lines: recipe.body.iter().map(Line::new).collect(),
      parameters: recipe.parameters.iter().map(Parameter::new).collect(),
      private: recipe.private,
      quiet: recipe.quiet,
      shebang: recipe.shebang,
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Parameter {
  pub default: Option<Expression>,
  pub kind: ParameterKind,
  pub name: String,
}

impl Parameter {
  fn new(parameter: &full::Parameter) -> Self {
    Self {
      kind: ParameterKind::new(parameter.kind),
      name: parameter.name.lexeme().to_owned(),
      default: parameter.default.as_ref().map(Expression::new),
    }
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ParameterKind {
  Plus,
  Singular,
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
  fn new(line: &full::Line) -> Self {
    Self {
      fragments: line.fragments.iter().map(Fragment::new).collect(),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub enum Fragment {
  Expression { expression: Expression },
  Text { text: String },
}

impl Fragment {
  fn new(fragment: &full::Fragment) -> Self {
    match fragment {
      full::Fragment::Text { token } => Self::Text {
        text: token.lexeme().to_owned(),
      },
      full::Fragment::Interpolation { expression } => Self::Expression {
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
  fn new(assignment: &full::Assignment) -> Self {
    Self {
      exported: assignment.export,
      expression: Expression::new(&assignment.value),
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub enum Expression {
  And {
    lhs: Box<Self>,
    rhs: Box<Self>,
  },
  Assert {
    condition: Condition,
    error: Box<Self>,
  },
  Backtick {
    command: String,
  },
  Call {
    name: String,
    arguments: Vec<Self>,
  },
  Concatenation {
    lhs: Box<Self>,
    rhs: Box<Self>,
  },
  Conditional {
    lhs: Box<Self>,
    rhs: Box<Self>,
    then: Box<Self>,
    otherwise: Box<Self>,
    operator: ConditionalOperator,
  },
  FormatString {
    start: String,
    expressions: Vec<(Self, String)>,
  },
  Join {
    lhs: Option<Box<Self>>,
    rhs: Box<Self>,
  },
  Or {
    lhs: Box<Self>,
    rhs: Box<Self>,
  },
  String {
    text: String,
  },
  Variable {
    name: String,
  },
}

impl Expression {
  fn new(expression: &full::Expression) -> Self {
    use full::Expression::*;
    match expression {
      And { lhs, rhs } => Self::And {
        lhs: Self::new(lhs).into(),
        rhs: Self::new(rhs).into(),
      },
      Assert {
        condition: full::Condition { lhs, rhs, operator },
        error,
        ..
      } => Self::Assert {
        condition: Condition {
          lhs: Box::new(Self::new(lhs)),
          rhs: Box::new(Self::new(rhs)),
          operator: ConditionalOperator::new(*operator),
        },
        error: Box::new(Self::new(error)),
      },
      Backtick { contents, .. } => Self::Backtick {
        command: (*contents).clone(),
      },
      Call { thunk } => match thunk {
        full::Thunk::Nullary { name, .. } => Self::Call {
          name: name.lexeme().to_owned(),
          arguments: Vec::new(),
        },
        full::Thunk::Unary { name, arg, .. } => Self::Call {
          name: name.lexeme().to_owned(),
          arguments: vec![Self::new(arg)],
        },
        full::Thunk::UnaryOpt {
          name,
          args: (a, opt_b),
          ..
        } => {
          let mut arguments = Vec::new();
          if let Some(b) = opt_b.as_ref() {
            arguments.push(Self::new(b));
          }
          arguments.push(Self::new(a));
          Self::Call {
            name: name.lexeme().to_owned(),
            arguments,
          }
        }
        full::Thunk::UnaryPlus {
          name,
          args: (a, rest),
          ..
        } => {
          let mut arguments = vec![Self::new(a)];
          for arg in rest {
            arguments.push(Self::new(arg));
          }
          Self::Call {
            name: name.lexeme().to_owned(),
            arguments,
          }
        }
        full::Thunk::Binary {
          name, args: [a, b], ..
        } => Self::Call {
          name: name.lexeme().to_owned(),
          arguments: vec![Self::new(a), Self::new(b)],
        },
        full::Thunk::BinaryPlus {
          name,
          args: ([a, b], rest),
          ..
        } => {
          let mut arguments = vec![Self::new(a), Self::new(b)];
          for arg in rest {
            arguments.push(Self::new(arg));
          }
          Self::Call {
            name: name.lexeme().to_owned(),
            arguments,
          }
        }
        full::Thunk::Ternary {
          name,
          args: [a, b, c],
          ..
        } => Self::Call {
          name: name.lexeme().to_owned(),
          arguments: vec![Self::new(a), Self::new(b), Self::new(c)],
        },
      },
      Concatenation { lhs, rhs } => Self::Concatenation {
        lhs: Self::new(lhs).into(),
        rhs: Self::new(rhs).into(),
      },
      Conditional {
        condition: full::Condition { lhs, rhs, operator },
        otherwise,
        then,
      } => Self::Conditional {
        lhs: Self::new(lhs).into(),
        operator: ConditionalOperator::new(*operator),
        otherwise: Self::new(otherwise).into(),
        rhs: Self::new(rhs).into(),
        then: Self::new(then).into(),
      },
      FormatString { start, expressions } => Self::FormatString {
        start: start.cooked.clone(),
        expressions: expressions
          .iter()
          .map(|(expression, string)| (Self::new(expression), string.cooked.clone()))
          .collect(),
      },
      Group { contents } => Self::new(contents),
      Join { lhs, rhs } => Self::Join {
        lhs: lhs.as_ref().map(|lhs| Self::new(lhs).into()),
        rhs: Self::new(rhs).into(),
      },
      Or { lhs, rhs } => Self::Or {
        lhs: Self::new(lhs).into(),
        rhs: Self::new(rhs).into(),
      },
      StringLiteral { string_literal } => Self::String {
        text: string_literal.cooked.clone(),
      },
      Variable { name, .. } => Self::Variable {
        name: name.lexeme().to_owned(),
      },
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Condition {
  lhs: Box<Expression>,
  operator: ConditionalOperator,
  rhs: Box<Expression>,
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub enum ConditionalOperator {
  Equality,
  Inequality,
  RegexMatch,
  RegexMismatch,
}

impl ConditionalOperator {
  fn new(operator: full::ConditionalOperator) -> Self {
    match operator {
      full::ConditionalOperator::Equality => Self::Equality,
      full::ConditionalOperator::Inequality => Self::Inequality,
      full::ConditionalOperator::RegexMatch => Self::RegexMatch,
      full::ConditionalOperator::RegexMismatch => Self::RegexMismatch,
    }
  }
}

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone)]
pub struct Dependency {
  pub arguments: Vec<Expression>,
  pub recipe: String,
}

impl Dependency {
  fn new(dependency: &full::Dependency) -> Self {
    let mut arguments = Vec::new();
    for group in &dependency.arguments {
      for argument in group {
        arguments.push(Expression::new(argument));
      }
    }

    Self {
      recipe: dependency.recipe.name().to_owned(),
      arguments,
    }
  }
}
