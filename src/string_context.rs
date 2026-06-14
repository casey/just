use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum StringContext<'src> {
  Assert,
  Concatenation,
  ConfirmPrompt,
  DotenvFilename,
  DotenvPath,
  EnvKey,
  EnvValue,
  Function { name: Name<'src> },
  Interpolation,
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
      Self::Assert => write!(f, "used as an `assert` failure message"),
      Self::Concatenation => write!(f, "used as an operand of `+`"),
      Self::ConfirmPrompt => write!(f, "used as a `[confirm]` prompt"),
      Self::DotenvFilename => write!(f, "assigned to setting `dotenv-filename`"),
      Self::DotenvPath => write!(f, "assigned to setting `dotenv-path`"),
      Self::EnvKey => write!(f, "used as an `env` attribute name"),
      Self::EnvValue => write!(f, "used as an `env` attribute value"),
      Self::Function { name } => write!(f, "passed to function `{name}`"),
      Self::Interpolation => write!(f, "used in an interpolation"),
      Self::Join => write!(f, "used as an operand of `/`"),
      Self::Regex => write!(f, "used as a regular expression"),
      Self::ScriptInterpreter => write!(f, "assigned to setting `script-interpreter`"),
      Self::Shell => write!(f, "assigned to setting `shell`"),
      Self::Tempdir => write!(f, "assigned to setting `tempdir`"),
      Self::WindowsShell => write!(f, "assigned to setting `windows-shell`"),
      Self::WorkingDirectoryAttribute => write!(f, "used as a `[working-directory]` attribute"),
      Self::WorkingDirectorySetting => write!(f, "assigned to setting `working-directory`"),
    }
  }
}
