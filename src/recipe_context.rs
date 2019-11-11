use crate::common::*;

pub(crate) struct RecipeContext<'a> {
  pub(crate) config: &'a Config,
  pub(crate) scope: BTreeMap<&'a str, (bool, String)>,
  pub(crate) working_directory: &'a Path,
  pub(crate) settings: &'a Settings<'a>,
}
