use super::*;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub(crate) enum Word<'src> {
  Expression(Expression<'src>),
  Text(String),
}
