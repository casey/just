use super::*;

pub(crate) struct ArgAttribute<'src> {
  pub(crate) flag: bool,
  pub(crate) long: Option<String>,
  pub(crate) max: Option<(Name<'src>, u64)>,
  pub(crate) multiple: bool,
  pub(crate) name: Token<'src>,
  pub(crate) short: Option<char>,
  pub(crate) value: Option<Expression<'src>>,
}
