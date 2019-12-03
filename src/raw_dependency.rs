use crate::common::*;

#[derive(PartialEq, Debug)]
pub(crate) struct RawDependency<'src> {
  pub(crate) recipe: Name<'src>,
}
