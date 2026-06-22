use super::*;

#[derive(Serialize)]
pub(crate) struct CacheKey<'a> {
  pub(crate) body: String,
  pub(crate) environment: &'a Environment,
  pub(crate) executor: &'a Executor<'a>,
  pub(crate) positional: Option<&'a [String]>,
  pub(crate) recipe: &'a Modulepath,
  pub(crate) working_directory: Option<&'a Path>,
}
