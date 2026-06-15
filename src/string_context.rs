use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum StringContext<'src> {
  Concatenation,
  EnvKey(Name<'src>),
  Function(Name<'src>),
  Join,
  Regex,
  Setting(Name<'src>),
  WorkingDirectoryAttribute(Name<'src>),
}

impl<'src> StringContext<'src> {
  pub(crate) fn token(&self) -> Option<Token<'src>> {
    match self {
      Self::EnvKey(name)
      | Self::Function(name)
      | Self::Setting(name)
      | Self::WorkingDirectoryAttribute(name) => Some(name.token),
      _ => None,
    }
  }
}

impl Display for StringContext<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Concatenation => write!(f, "used as `+` operand"),
      Self::EnvKey(_) => write!(f, "used as `env` attribute name"),
      Self::Function(name) => write!(f, "passed to `{name}()`"),
      Self::Join => write!(f, "used as `/` operand"),
      Self::Regex => write!(f, "used as regular expression"),
      Self::Setting(name) => write!(f, "assigned to `{name}` setting"),
      Self::WorkingDirectoryAttribute(_) => {
        write!(f, "used as a `[working-directory]` attribute")
      }
    }
  }
}
