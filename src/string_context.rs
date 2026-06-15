use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum StringContext<'src> {
  Concatenation,
  EnvKey,
  Function { name: Name<'src> },
  Join,
  Regex,
  Setting(Name<'src>),
  WorkingDirectoryAttribute,
}

impl<'src> StringContext<'src> {
  pub(crate) fn token(&self) -> Option<Token<'src>> {
    match self {
      Self::Function { name } | Self::Setting(name) => Some(name.token),
      _ => None,
    }
  }
}

impl Display for StringContext<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Concatenation => write!(f, "used as `+` operand"),
      Self::EnvKey => write!(f, "used as `env` attribute name"),
      Self::Function { name } => write!(f, "passed to `{name}()`"),
      Self::Join => write!(f, "used as `/` operand"),
      Self::Regex => write!(f, "used as regular expression"),
      Self::Setting(name) => write!(f, "assigned to `{name}` setting"),
      Self::WorkingDirectoryAttribute => write!(f, "used as a `[working-directory]` attribute"),
    }
  }
}
