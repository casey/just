use super::*;

#[derive(Serialize)]
pub(crate) struct CacheKey<'a> {
  pub(crate) environment: &'a BTreeMap<String, Option<String>>,
  pub(crate) executor: &'a Executor<'a>,
  pub(crate) lines: &'a [String],
  pub(crate) positional: Option<&'a [String]>,
  pub(crate) recipe: &'a Modulepath,
  pub(crate) working_directory: Option<&'a Path>,
}
