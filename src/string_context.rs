use super::*;

#[derive(Debug)]
pub(crate) enum StringContext<'src> {
  Function { name: Name<'src> },
}

impl Display for StringContext<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Function { name } => write!(f, "passed to function {name}"),
    }
  }
}
