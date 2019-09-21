use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) enum Fragment<'a> {
  Text { text: Token<'a> },
  Expression { expression: Expression<'a> },
}

impl<'a> Fragment<'a> {
  pub(crate) fn continuation(&self) -> bool {
    match *self {
      Fragment::Text { ref text } => text.lexeme().ends_with('\\'),
      _ => false,
    }
  }
}
