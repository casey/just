use crate::common::*;

#[derive(Debug, Clone)]
pub(crate) enum Setting<'src> {
  AllowDuplicateRecipes(bool),
  DotenvLoad(bool),
  Export(bool),
  PositionalArguments(bool),
  Shell(Shell<'src>),
  WindowsPowerShell(bool),
}

impl<'src> Display for Setting<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match self {
      Setting::AllowDuplicateRecipes(value)
      | Setting::DotenvLoad(value)
      | Setting::Export(value)
      | Setting::PositionalArguments(value)
      | Setting::WindowsPowerShell(value) => write!(f, "{}", value),
      Setting::Shell(shell) => write!(f, "{}", shell),
    }
  }
}
