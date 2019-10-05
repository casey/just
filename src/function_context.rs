use crate::common::*;

pub(crate) struct FunctionContext<'a> {
  pub(crate) invocation_directory: &'a Result<PathBuf, String>,
  pub(crate) justfile_directory: &'a Result<PathBuf, String>,
  pub(crate) dotenv: &'a BTreeMap<String, String>,
}
