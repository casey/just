use crate::common::*;

pub(crate) struct FunctionContext<'run> {
  pub(crate) invocation_directory: &'run Path,
  pub(crate) working_directory: &'run Path,
  pub(crate) dotenv: &'run BTreeMap<String, String>,
}
