use crate::common::*;

pub(crate) struct RecipeContext<'a> {
  pub(crate) invocation_directory: &'a Result<PathBuf, String>,
  pub(crate) config: &'a Config<'a>,
  pub(crate) scope: BTreeMap<&'a str, String>,
}
