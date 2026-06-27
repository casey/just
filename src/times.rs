use super::*;

pub(crate) struct Times(pub(crate) usize);

impl Display for Times {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.0 {
      0 => write!(f, "0 times"),
      1 => write!(f, "once"),
      2 => write!(f, "twice"),
      3 => write!(f, "thrice"),
      n => write!(f, "{n} times"),
    }
  }
}
