use super::*;

#[derive(Debug, Clone)]
pub(crate) enum Setting<'src> {
  AllowDuplicateRecipes(bool),
  AllowDuplicateVariables(bool),
  DotenvFilename(Expression<'src>),
  DotenvLoad(bool),
  DotenvOverride(bool),
  DotenvPath(Expression<'src>),
  DotenvRequired(bool),
  Export(bool),
  Fallback(bool),
  IgnoreComments(bool),
  NoExitMessage(bool),
  PositionalArguments(bool),
  Quiet(bool),
  ScriptInterpreter(Interpreter),
  Shell(Interpreter),
  Tempdir(Expression<'src>),
  Unstable(bool),
  WindowsPowerShell(bool),
  WindowsShell(Interpreter),
  WorkingDirectory(Expression<'src>),
}

impl<'src> Setting<'src> {
  pub(crate) fn expression(&self) -> Option<&Expression<'src>> {
    match self {
      Self::DotenvFilename(value)
      | Self::DotenvPath(value)
      | Self::Tempdir(value)
      | Self::WorkingDirectory(value) => Some(value),
      _ => None,
    }
  }
}

impl Display for Setting<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::AllowDuplicateRecipes(value)
      | Self::AllowDuplicateVariables(value)
      | Self::DotenvLoad(value)
      | Self::DotenvOverride(value)
      | Self::DotenvRequired(value)
      | Self::Export(value)
      | Self::Fallback(value)
      | Self::IgnoreComments(value)
      | Self::NoExitMessage(value)
      | Self::PositionalArguments(value)
      | Self::Quiet(value)
      | Self::Unstable(value)
      | Self::WindowsPowerShell(value) => write!(f, "{value}"),
      Self::DotenvFilename(value)
      | Self::DotenvPath(value)
      | Self::Tempdir(value)
      | Self::WorkingDirectory(value) => {
        write!(f, "{value}")
      }
      Self::ScriptInterpreter(shell) | Self::Shell(shell) | Self::WindowsShell(shell) => {
        write!(f, "[{shell}]")
      }
    }
  }
}
