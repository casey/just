use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Suggestion<'src> {
  pub(crate) name: &'src str,
  pub(crate) target: Option<&'src str>,
}

impl<'src> Suggestion<'src> {
  pub fn find_suggestion(
    input: &str,
    candidates: impl Iterator<Item = Suggestion<'src>>,
  ) -> Option<Suggestion<'src>> {
    candidates
      .map(|suggestion| (edit_distance(input, suggestion.name), suggestion))
      .filter(|(distance, _suggestion)| *distance < 3)
      .min_by_key(|(distance, _suggestion)| *distance)
      .map(|(_distance, suggestion)| suggestion)
  }
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
