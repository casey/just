use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub(crate) enum UnstableFeature {
  CachedRecipes,
  ListsSetting,
  UserDefinedFunction,
}

impl Display for UnstableFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::CachedRecipes => write!(f, "cached recipes are currently unstable"),
      Self::ListsSetting => write!(f, "the `lists` setting is currently unstable"),
      Self::UserDefinedFunction => {
        write!(f, "user-defined functions are currently unstable")
      }
    }
  }
}
