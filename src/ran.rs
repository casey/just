use super::*;

#[derive(Default)]
pub(crate) struct Ran(Mutex<BTreeMap<Modulepath, BTreeMap<Vec<Value>, Arc<Mutex<bool>>>>>);

impl Ran {
  pub(crate) fn mutex(&self, recipe: &Recipe, arguments: &[Value]) -> Arc<Mutex<bool>> {
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

  pub(crate) fn new() -> Self {
    Self::default()
  }
}
