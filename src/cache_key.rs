use super::*;

#[derive(Serialize)]
pub(crate) struct CacheKey<'a> {
  pub(crate) body: &'a [String],
  pub(crate) environment: &'a Environment,
  pub(crate) executor: &'a Executor<'a>,
  pub(crate) inputs: Option<&'a BTreeMap<String, String>>,
  pub(crate) positional: Option<&'a [String]>,
  pub(crate) recipe: &'a Modulepath,
  pub(crate) working_directory: Option<&'a Path>,
}
