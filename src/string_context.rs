use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum StringContext<'src> {
  ArgPattern(Name<'src>),
  EnvKey(Name<'src>),
  Function(Name<'src>),
  Setting(Name<'src>),
  WorkingDirectoryAttribute(Name<'src>),
}

impl<'src> StringContext<'src> {
  pub(crate) fn token(&self) -> Token<'src> {
    match self {
      Self::ArgPattern(name)
      | Self::EnvKey(name)
      | Self::Function(name)
      | Self::Setting(name)
      | Self::WorkingDirectoryAttribute(name) => name.token,
    }
  }
}

impl Display for StringContext<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::ArgPattern(_) => write!(f, "used as `arg` attribute pattern"),
      Self::EnvKey(_) => write!(f, "used as `env` attribute name"),
      Self::Function(name) => write!(f, "passed to `{name}()`"),
      Self::Setting(name) => write!(f, "assigned to `{name}` setting"),
      Self::WorkingDirectoryAttribute(_) => {
        write!(f, "used as a `[working-directory]` attribute")
      }
    }
  }
}
