use super::*;

const DEFAULT_DOTENV_FILENAME: &str = ".env";

pub(crate) fn load_dotenv(
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  if !settings.dotenv_load.unwrap_or(false)
    && config.dotenv_filename.is_none()
    && config.dotenv_path.is_none()
  {
    return Ok(BTreeMap::new());
  }

  if let Some(path) = &config.dotenv_path {
    return load_from_file(path);
  }

  let filename = config
    .dotenv_filename
    .as_deref()
    .unwrap_or(DEFAULT_DOTENV_FILENAME)
    .to_owned();

  for directory in working_directory.ancestors() {
    let path = directory.join(&filename);
    if path.is_file() {
      return load_from_file(&path);
    }
  }

  Ok(BTreeMap::new())
}

fn load_from_file(path: &Path) -> RunResult<'static, BTreeMap<String, String>> {
  // `dotenv::from_path_iter` should eventually be un-deprecated, see:
  // https://github.com/dotenv-rs/dotenv/issues/13
  #![allow(deprecated)]

  let iter = dotenv::from_path_iter(path)?;
  let mut dotenv = BTreeMap::new();
  for result in iter {
    let (key, value) = result?;
    if env::var_os(&key).is_none() {
      dotenv.insert(key, value);
    }
  }
  Ok(dotenv)
}
