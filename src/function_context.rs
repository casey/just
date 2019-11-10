use crate::common::*;

pub(crate) struct FunctionContext<'a> {
  pub(crate) invocation_directory: &'a Path,
  pub(crate) working_directory: &'a Path,
  pub(crate) dotenv: &'a BTreeMap<String, String>,
}
