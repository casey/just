use super::*;

#[derive(Default)]
pub(crate) struct Ran(Mutex<BTreeMap<Modulepath, BTreeMap<Vec<Vec<String>>, Arc<Mutex<bool>>>>>);

impl Ran {
  pub(crate) fn mutex(&self, recipe: &Recipe, arguments: &[Vec<String>]) -> Arc<Mutex<bool>> {
    self
      .0
      .lock()
      .unwrap()
      .entry(recipe.recipe_path().clone())
      .or_default()
      .entry(arguments.into())
      .or_default()
      .clone()
  }
}
