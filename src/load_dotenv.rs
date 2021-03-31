use crate::common::*;

pub(crate) fn load_dotenv(
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  // `dotenv::from_path_iter` should eventually be un-deprecated, see:
  // https://github.com/dotenv-rs/dotenv/issues/13
  #![allow(deprecated)]

  if !settings.dotenv_load.unwrap_or(true) {
    return Ok(BTreeMap::new());
  }

  for directory in working_directory.ancestors() {
    let path = directory.join(".env");

    if path.is_file() {
      if settings.dotenv_load.is_none() && config.verbosity.loud() {
        if config.color.stderr().active() {
          eprintln!("{:#}", Warning::DotenvLoad);
        } else {
          eprintln!("{}", Warning::DotenvLoad);
        }
      }

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
