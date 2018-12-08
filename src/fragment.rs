use common::*;

#[derive(PartialEq, Debug)]
pub enum Fragment<'a> {
  Text { text: Token<'a> },
  Expression { expression: Expression<'a> },
}

impl<'a> Fragment<'a> {
  pub fn continuation(&self) -> bool {
    match *self {
      Fragment::Text { ref text } => text.lexeme.ends_with('\\'),
      _ => false,
    }
  }
}
