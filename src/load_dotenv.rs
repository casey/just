use super::*;

pub(crate) fn load_dotenv(
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  let dotenv_filename = config
    .dotenv_filename
    .as_ref()
    .or(settings.dotenv_filename.as_ref());

  let dotenv_path = config
    .dotenv_path
    .as_ref()
    .or(settings.dotenv_path.as_ref());

  if !settings.dotenv_load
    && !settings.dotenv_override
    && !settings.dotenv_required
    && dotenv_filename.is_none()
    && dotenv_path.is_none()
  {
    return Ok(BTreeMap::new());
  }

  if let Some(path) = dotenv_path {
    let path = working_directory.join(path);
    if filesystem::is_file(&path)? {
      return load_from_file(&path, settings);
    }
  }

  let filename = dotenv_filename.map_or(".env", |s| s.as_str());

  for directory in working_directory.ancestors() {
    let path = directory.join(filename);
    if filesystem::is_file(&path)? {
      return load_from_file(&path, settings);
    }
  }

  if settings.dotenv_required {
    Err(Error::DotenvRequired)
  } else {
    Ok(BTreeMap::new())
  }
}

fn load_from_file(
  path: &Path,
  settings: &Settings,
) -> RunResult<'static, BTreeMap<String, String>> {
  let iter = dotenvy::from_path_iter(path)?;
  let mut dotenv = BTreeMap::new();
  for result in iter {
    let (key, value) = result?;
    if settings.dotenv_override || env::var_os(&key).is_none() {
      dotenv.insert(key, value);
    }
  }
  Ok(dotenv)
}
