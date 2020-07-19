use crate::common::*;

pub(crate) fn load_dotenv(
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  // `dotenv::from_path_iter` should eventually be un-deprecated, see:
  // https://github.com/dotenv-rs/dotenv/issues/13
  #![allow(deprecated)]
  for directory in working_directory.ancestors() {
    let path = directory.join(".env");

    if path.is_file() {
      let iter = dotenv::from_path_iter(&path)?;
      let mut dotenv = BTreeMap::new();
      for result in iter {
        let (key, value) = result?;
        if env::var_os(&key).is_none() {
          dotenv.insert(key, value);
        }
      }
      return Ok(dotenv);
    }
  }

  Ok(BTreeMap::new())
}
