use crate::common::*;

#[derive(Debug)]
pub(crate) enum Setting<'src> {
  DotenvLoad(bool),
  Export(bool),
  PositionalArguments(bool),
  Shell(Shell<'src>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct Shell<'src> {
  pub(crate) command:   StringLiteral<'src>,
  pub(crate) arguments: Vec<StringLiteral<'src>>,
}
