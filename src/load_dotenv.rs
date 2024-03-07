use super::*;

const DEFAULT_DOTENV_FILENAME: &str = ".env";

pub(crate) fn load_dotenv(
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  let dotenv_filename: &Vec<String> = if !config.dotenv_filename.is_empty() {
    config.dotenv_filename.as_ref()
  } else {
    settings.dotenv_filename.as_ref()
  };

  let dotenv_path = config
    .dotenv_path
    .as_ref()
    .or(settings.dotenv_path.as_ref());

  if !settings.dotenv_load.unwrap_or_default()
    && dotenv_filename.is_empty()
    && dotenv_path.is_none()
  {
    return Ok(BTreeMap::new());
  }

  if let Some(path) = dotenv_path {
    return load_from_file(path);
  }

  let default_files = vec![DEFAULT_DOTENV_FILENAME.to_owned()];
  let filenames = if dotenv_filename.is_empty() {
    &default_files
  } else {
    dotenv_filename
  };

  let mut envs = BTreeMap::new();
  for filename in filenames {
    for directory in working_directory.ancestors() {
      let path = directory.join(filename.clone());
      if path.is_file() {
        if let Ok(mut file_envs) = load_from_file(&path) {
          envs.append(&mut file_envs);
          break;
        }
      }
    }
  }

  Ok(envs)
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
