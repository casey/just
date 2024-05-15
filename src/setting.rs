use super::*;

#[derive(Debug, Clone)]
pub(crate) enum Setting<'src> {
  AllowDuplicateRecipes(bool),
  AllowDuplicateVariables(bool),
  DotenvFilename(String),
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
      Self::AllowDuplicateRecipes(value)
      | Self::AllowDuplicateVariables(value)
      | Self::DotenvLoad(value)
      | Self::Export(value)
      | Self::Fallback(value)
      | Self::IgnoreComments(value)
      | Self::PositionalArguments(value)
      | Self::Quiet(value)
      | Self::WindowsPowerShell(value) => write!(f, "{value}"),
      Self::Shell(shell) | Self::WindowsShell(shell) => write!(f, "{shell}"),
      Self::DotenvFilename(value) | Self::DotenvPath(value) | Self::Tempdir(value) => {
        write!(f, "{value:?}")
      }
    }
  }
}
