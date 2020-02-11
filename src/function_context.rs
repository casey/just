use crate::common::*;

pub(crate) struct FunctionContext<'run> {
  pub(crate) dotenv:               &'run BTreeMap<String, String>,
  pub(crate) invocation_directory: &'run Path,
  pub(crate) search:               &'run Search,
}
