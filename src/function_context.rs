use crate::common::*;

pub struct FunctionContext<'a> {
  pub invocation_directory: &'a Result<PathBuf, String>,
  pub dotenv: &'a BTreeMap<String, String>,
}
