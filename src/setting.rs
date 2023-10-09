use super::*;

#[derive(Debug, Clone)]
pub(crate) enum Setting<'src> {
  AllowDuplicateRecipes(bool),
  DotenvLoad(bool),
  Export(bool),
  Fallback(bool),
  IgnoreComments(bool),
  PositionalArguments(bool),
  Shell(Shell<'src>),
  Tempdir(String),
  WindowsPowerShell(bool),
  WindowsShell(Shell<'src>),
}

impl<'src> Display for Setting<'src> {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    match self {
      Setting::AllowDuplicateRecipes(value)
      | Setting::DotenvLoad(value)
      | Setting::Export(value)
      | Setting::Fallback(value)
      | Setting::IgnoreComments(value)
      | Setting::PositionalArguments(value)
      | Setting::WindowsPowerShell(value) => write!(f, "{value}"),
      Setting::Shell(shell) | Setting::WindowsShell(shell) => write!(f, "{shell}"),
      Setting::Tempdir(tempdir) => {
        write!(f, "{tempdir:?}")
      }
    }
  }
}
