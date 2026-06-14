use super::*;

#[derive(Debug)]
pub(crate) enum StringContext {
  Function { name: String },
}

impl Display for StringContext {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Function { name } => write!(f, "passed to function {name}"),
    }
  }
}
