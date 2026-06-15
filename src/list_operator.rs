use super::*;

#[derive(Debug)]
pub(crate) enum ListOperator {
  Concatenate,
  Join,
}

impl ListOperator {
  pub(crate) fn separator(self) -> &'static str {
    match self {
      Self::Concatenate => "",
      Self::Join => "/",
    }
  }
}

impl Display for ListOperator {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Concatenate => write!(f, "+"),
      Self::Join => write!(f, "/"),
    }
  }
}
