use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) struct UnresolvedDependency<'src> {
  pub(crate) recipe:    Name<'src>,
  pub(crate) arguments: Vec<Expression<'src>>,
}
