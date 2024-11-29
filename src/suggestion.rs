use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Suggestion<'src> {
  pub(crate) name: &'src str,
  pub(crate) target: Option<&'src str>,
}

impl Display for Suggestion<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "Did you mean `{}`", self.name)?;
    if let Some(target) = self.target {
      write!(f, ", an alias for `{target}`")?;
    }
    write!(f, "?")
  }
}
