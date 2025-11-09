use super::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FlagArity {
  Switch,
  WithValue,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct FlagSpec<'src> {
  pub(crate) arity: FlagArity,
  pub(crate) default: Option<Expression<'src>>,
  pub(crate) name: Name<'src>,
}
