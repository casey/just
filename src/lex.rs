use super::*;

pub(crate) struct Lex<'src> {
  pub(crate) dropped_comment: Option<Token<'src>>,
  pub(crate) tokens: Vec<Token<'src>>,
}
