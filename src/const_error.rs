use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum ConstError<'src> {
  Backtick(Token<'src>),
  FunctionCall(Name<'src>),
  Variable(Name<'src>),
}

impl<'src> ConstError<'src> {
  pub(crate) fn context(self) -> Token<'src> {
    match self {
      Self::Backtick(token) => token,
      Self::FunctionCall(name) | Self::Variable(name) => name.token,
    }
  }
}

impl Display for ConstError<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Backtick(_) => write!(f, "Cannot call backticks in const context"),
      Self::FunctionCall(_) => write!(f, "Cannot call functions in const context"),
      Self::Variable(name) => write!(
        f,
        "Cannot access non-const variable `{name}` in const context"
      ),
    }
  }
}
