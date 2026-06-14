use super::*;

#[derive(Copy, Clone, Debug)]
pub(crate) enum RestrictedFunction {
  List(ListFeature),
  Unstable(UnstableFeature),
}
