use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum StringContext<'src> {
  Concatenation,
  DotenvFilename,
  DotenvPath,
  EnvKey,
  Function { name: Name<'src> },
  Join,
  Regex,
  ScriptInterpreter,
  Shell,
  Tempdir,
  WindowsShell,
  WorkingDirectoryAttribute,
  WorkingDirectorySetting,
}

impl Display for StringContext<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Concatenation => write!(f, "used as `+` operand"),
      Self::DotenvFilename => write!(f, "assigned to `dotenv-filename` setting"),
      Self::DotenvPath => write!(f, "assigned to `dotenv-path` setting"),
      Self::EnvKey => write!(f, "used as `env` attribute name"),
      Self::Function { name } => write!(f, "passed to `{name}()`"),
      Self::Join => write!(f, "used as `/` operand"),
      Self::Regex => write!(f, "used as regular expression"),
      Self::ScriptInterpreter => write!(f, "assigned to `script-interpreter` setting"),
      Self::Shell => write!(f, "assigned to `shell` setting"),
      Self::Tempdir => write!(f, "assigned to `tempdir` setting"),
      Self::WindowsShell => write!(f, "assigned to `windows-shell` setting"),
      Self::WorkingDirectoryAttribute => write!(f, "used as a `[working-directory]` attribute"),
      Self::WorkingDirectorySetting => write!(f, "assigned to `working-directory` setting"),
    }
  }
}
