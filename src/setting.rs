use super::*;

#[derive(Debug, Clone)]
pub(crate) enum Setting<'src> {
  AllowDuplicateRecipes(bool),
  DotenvFilename(Vec<String>),
  DotenvLoad(bool),
  DotenvPath(String),
  Export(bool),
  Fallback(bool),
  IgnoreComments(bool),
  PositionalArguments(bool),
  Quiet(bool),
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
      | Setting::Quiet(value)
      | Setting::WindowsPowerShell(value) => write!(f, "{value}"),
      Setting::Shell(shell) | Setting::WindowsShell(shell) => write!(f, "{shell}"),
      Setting::DotenvFilename(value) => {
        if value.is_empty() {
          write!(f, "{value:?}")
        } else {
          write!(f, "{}", value.join(","))
        }
      }
      Setting::DotenvPath(value) | Setting::Tempdir(value) => {
        write!(f, "{value:?}")
      }
    }
  }
}
