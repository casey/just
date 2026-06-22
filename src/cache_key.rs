use super::*;

#[derive(Serialize)]
pub(crate) struct CacheKey<'a> {
  pub(crate) body: &'a [String],
  pub(crate) environment: &'a Environment,
  pub(crate) executor: &'a Executor<'a>,
  pub(crate) extra: Option<Value>,
  pub(crate) inputs: Option<BTreeMap<String, blake3::Hash>>,
  pub(crate) positional: Option<&'a [String]>,
  pub(crate) recipe: &'a Modulepath,
  pub(crate) working_directory: Option<&'a Path>,
}
