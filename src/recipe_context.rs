use crate::common::*;

pub(crate) struct RecipeContext<'a> {
  pub(crate) config: &'a Config<'a>,
  pub(crate) scope: BTreeMap<&'a str, (bool, String)>,
}
