use super::*;

#[derive(Copy, Clone, Debug)]
pub(crate) enum FunctionFeature {
  List(ListFeature),
  Unstable(UnstableFeature),
}
