use crate::common::*;

fn load_from_file(
  config: &Config,
  settings: &Settings,
  path: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  // `dotenv::from_path_iter` should eventually be un-deprecated, see:
  // https://github.com/dotenv-rs/dotenv/issues/13
  #![allow(deprecated)]

  if settings.dotenv_load.is_none()
    && config.verbosity.loud()
    && !std::env::var_os("JUST_SUPPRESS_DOTENV_LOAD_WARNING")
      .map(|val| val.as_os_str().to_str() == Some("1"))
      .unwrap_or(false)
  {
    eprintln!(
      "{}",
      Warning::DotenvLoad.color_display(config.color.stderr())
    );
  }

  let iter = dotenv::from_path_iter(&path)?;
  let mut dotenv = BTreeMap::new();
  for result in iter {
    let (key, value) = result?;
    if env::var_os(&key).is_none() {
      dotenv.insert(key, value);
    }
  }
  Ok(dotenv)
}

pub(crate) fn load_dotenv(
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  if !settings.dotenv_load.unwrap_or(true) {
    return Ok(BTreeMap::new());
  }

  // look directly for the environment file if specified by path
  let path = working_directory.join(&config.dotenv_path);
  if path.is_file() {
    return load_from_file(config, settings, &path);
  }

  // search upward for the environment file if specified by filename
  for directory in working_directory.ancestors() {
    let path = directory.join(&config.dotenv_filename);
    if path.is_file() {
      return load_from_file(config, settings, &path);
    }
  }

  Ok(BTreeMap::new())
}
