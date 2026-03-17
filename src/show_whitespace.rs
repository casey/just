use super::*;

/// String wrapper that uses nonblank characters to display spaces and tabs
pub(crate) struct ShowWhitespace<'str>(pub &'str str);

impl Display for ShowWhitespace<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for c in self.0.chars() {
      match c {
        '\t' => write!(f, "␉")?,
        ' ' => write!(f, "␠")?,
        _ => write!(f, "{c}")?,
      }
    }

    Ok(())
  }
}
