use super::*;

#[derive(Default)]
pub(crate) struct Ran<'src>(BTreeMap<Namepath<'src>, BTreeSet<Vec<String>>>);

impl<'src> Ran<'src> {
  pub(crate) fn has_run(&self, recipe: &Namepath<'src>, arguments: &[String]) -> bool {
    self
      .0
      .get(recipe)
      .map(|ran| ran.contains(arguments))
      .unwrap_or_default()
  }

  pub(crate) fn ran(&mut self, recipe: &Namepath<'src>, arguments: Vec<String>) {
    self.0.entry(recipe.clone()).or_default().insert(arguments);
  }
}
