use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub(crate) enum UnstableFeature {
  CachedRecipes,
  FormatSubcommand,
  LogicalOperators,
  ScriptAttribute,
  ScriptInterpreterSetting,
  WhichFunction,
}

impl Display for UnstableFeature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::CachedRecipes => write!(f, "The [cached] attribute is currently unstable."),
      Self::FormatSubcommand => write!(f, "The `--fmt` command is currently unstable."),
      Self::LogicalOperators => write!(
        f,
        "The logical operators `&&` and `||` are currently unstable."
      ),
      Self::ScriptAttribute => write!(f, "The `[script]` attribute is currently unstable."),
      Self::ScriptInterpreterSetting => {
        write!(f, "The `script-interpreter` setting is currently unstable.")
      }
      Self::WhichFunction => write!(f, "The `which()` function is currently unstable."),
    }
  }
}
