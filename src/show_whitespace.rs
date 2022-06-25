use super::*;

/// String wrapper that uses nonblank characters to display spaces and tabs
pub struct ShowWhitespace<'str>(pub &'str str);

impl<'str> Display for ShowWhitespace<'str> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for c in self.0.chars() {
      match c {
        '\t' => write!(f, "␉")?,
        ' ' => write!(f, "␠")?,
        _ => write!(f, "{}", c)?,
      };
    }

    Ok(())
  }
}
