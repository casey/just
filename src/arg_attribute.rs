use super::*;

pub(crate) struct ArgAttribute<'src> {
  pub(crate) long: Option<String>,
  pub(crate) name: Token<'src>,
  pub(crate) pattern: Option<Pattern>,
  pub(crate) short: Option<char>,
}
