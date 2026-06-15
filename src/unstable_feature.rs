use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub(crate) enum UnstableFeature {
  ListsSetting,
  UserDefinedFunction,
}

impl Display for UnstableFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::ListsSetting => write!(f, "the `lists` setting is currently unstable"),
      Self::UserDefinedFunction => {
        write!(f, "user-defined functions are currently unstable")
      }
    }
  }
}
