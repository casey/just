use super::*;

const DEFAULT_DOTENV_FILENAME: &str = ".env";

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

  let dotenv_files = config.dotenv_files.as_ref().or_else(|| {
    if settings.dotenv_files.is_empty() {
      None
    } else {
      Some(settings.dotenv_files.as_ref())
    }
  });

  if !settings.dotenv_load.unwrap_or_default()
    && dotenv_filename.is_none()
    && dotenv_path.is_none()
    && dotenv_files.is_none()
  {
    return Ok(BTreeMap::new());
  }

  if let Some(path) = dotenv_path {
    return load_from_file(path);
  }

  let filename = dotenv_filename.map_or(DEFAULT_DOTENV_FILENAME, |s| s.as_str());

  for directory in working_directory.ancestors() {
    let path = directory.join(filename);
    if path.is_file() {
      return load_from_file(&path);
    }
  }

  let mut return_tree = BTreeMap::new();
  if let Some(dotenv_files) = dotenv_files {
    for path in dotenv_files.iter() {
      match load_from_file(&path) {
        Ok(mut env_tree) => return_tree.append(&mut env_tree),
        Err(error) => return Err(error),
      }
    }
  }
  Ok(return_tree)
}

fn load_from_file(path: &Path) -> RunResult<'static, BTreeMap<String, String>> {
  let iter = dotenvy::from_path_iter(path)?;
  let mut dotenv = BTreeMap::new();
  for result in iter {
    let (key, value) = result?;
    if env::var_os(&key).is_none() {
      dotenv.insert(key, value);
    }
  }
  Ok(dotenv)
}
