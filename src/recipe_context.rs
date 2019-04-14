use crate::common::*;

pub struct RecipeContext<'a> {
  pub invocation_directory: &'a Result<PathBuf, String>,
  pub configuration: &'a Configuration<'a>,
  pub scope: BTreeMap<&'a str, String>,
}
