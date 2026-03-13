use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum Switch {
  Long(String),
  Short(char),
}

impl Display for Switch {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match &self {
      Self::Long(long) => write!(f, "--{long}"),
      Self::Short(short) => write!(f, "-{short}"),
    }
  }
}
