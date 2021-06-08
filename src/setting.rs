use crate::common::*;

#[derive(Debug, Clone)]
pub(crate) enum Setting<'src> {
  DotenvLoad(bool),
  Export(bool),
  PositionalArguments(bool),
  Shell(Shell<'src>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Shell<'src> {
  pub(crate) command:   StringLiteral<'src>,
  pub(crate) arguments: Vec<StringLiteral<'src>>,
}

impl<'src> Display for Setting<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match self {
      Setting::DotenvLoad(value) => write!(f, "{}", value),
      Setting::Export(value) => write!(f, "{}", value),
      Setting::PositionalArguments(value) => write!(f, "{}", value),
      Setting::Shell(shell) => write!(f, "{}", shell),
    }
  }
}

impl<'src> Display for Shell<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(f, "[{}", self.command)?;

    for argument in &self.arguments {
      write!(f, ", {}", argument)?;
    }

    write!(f, "]")
  }
}
