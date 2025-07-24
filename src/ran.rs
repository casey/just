use super::*;

#[derive(Default)]
pub(crate) struct Ran(Mutex<BTreeMap<String, BTreeMap<Vec<String>, Arc<Mutex<bool>>>>>);

impl Ran {
  pub(crate) fn mutex(&self, recipe: &Recipe, arguments: &[String]) -> Arc<Mutex<bool>> {
    self
      .0
      .lock()
      .unwrap()
      .entry(recipe.namepath().into())
      .or_default()
      .entry(arguments.into())
      .or_default()
      .clone()
  }
}
