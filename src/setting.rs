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
  ScriptInterpreter(Interpreter<Expression<'src>>),
  Shell(Interpreter<Expression<'src>>),
  Tempdir(Expression<'src>),
  Unstable(bool),
  WindowsPowerShell(bool),
  WindowsShell(Interpreter<Expression<'src>>),
  WorkingDirectory(Expression<'src>),
}

impl<'src> Setting<'src> {
  pub(crate) fn expressions(&self) -> impl Iterator<Item = &Expression<'src>> {
    let first = match self {
      Self::DotenvFilename(value)
      | Self::DotenvPath(value)
      | Self::Tempdir(value)
      | Self::WorkingDirectory(value) => Some(value),
      Self::ScriptInterpreter(value) | Self::Shell(value) | Self::WindowsShell(value) => {
        Some(&value.command)
      }
      _ => None,
    };

    let rest = match self {
      Self::ScriptInterpreter(value) | Self::Shell(value) | Self::WindowsShell(value) => {
        value.arguments.as_slice()
      }
      _ => &[],
    };

    first.into_iter().chain(rest)
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
      Self::ScriptInterpreter(value) | Self::Shell(value) | Self::WindowsShell(value) => {
        write!(f, "[{value}]")
      }
    }
  }
}
