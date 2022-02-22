use crate::common::*;

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
  let mut dotenv: BTreeMap<String, String> = BTreeMap::new();

  if let Some(path) = &config.dotenv_path {
    return load_from_file(path, dotenv);
  }

  for filename in &settings.dotenv_filenames {
    let filename = config
      .dotenv_filename
      .as_deref()
      .unwrap_or(&filename.cooked)
      .to_owned();

    for directory in working_directory.ancestors() {
      let path = directory.join(&filename);
      if path.is_file() {
        dotenv = load_from_file(&path, dotenv)?;
      }
    }
  }
  return Ok(dotenv);
}

fn load_from_file(
  path: &Path,
  mut dotenv: BTreeMap<String, String>,
) -> RunResult<'static, BTreeMap<String, String>> {
  // `dotenv::from_path_iter` should eventually be un-deprecated, see:
  // https://github.com/dotenv-rs/dotenv/issues/13
  #![allow(deprecated)]

  let iter = dotenv::from_path_iter(&path)?;
  for result in iter {
    let (key, value) = result?;
    if env::var_os(&key).is_none() {
      dotenv.insert(key, value);
    }
  }
  return Ok(dotenv);
}
