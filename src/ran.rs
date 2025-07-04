use super::*;

#[derive(Default)]
pub(crate) struct Ran<'src>(
  Mutex<BTreeMap<Namepath<'src>, BTreeMap<Vec<String>, Arc<Mutex<bool>>>>>,
);

impl<'src> Ran<'src> {
  pub(crate) fn mutex(&self, recipe: &Namepath<'src>, arguments: &[String]) -> Arc<Mutex<bool>> {
    self
      .0
      .lock()
      .unwrap()
      .entry(recipe.clone())
      .or_default()
      .entry(arguments.into())
      .or_default()
      .clone()
  }
}
