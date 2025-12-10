use super::*;

pub(crate) struct ArgAttribute<'src> {
  pub(crate) name: Token<'src>,
  pub(crate) pattern: Option<Pattern>,
}
