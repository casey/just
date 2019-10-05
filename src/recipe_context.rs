use crate::common::*;

pub(crate) struct RecipeContext<'a> {
  pub(crate) invocation_directory: &'a Result<PathBuf, String>,
  pub(crate) justfile_directory: &'a Result<PathBuf, String>,
  pub(crate) configuration: &'a Configuration<'a>,
  pub(crate) scope: BTreeMap<&'a str, String>,
}
