use super::*;

pub(crate) fn load_dotenv(
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  let dotenv_filenames = if !config.dotenv_filename.is_empty() {
    config.dotenv_filename.as_slice()
  } else {
    settings.dotenv_filename.as_slice()
  };

  let dotenv_paths = if !config.dotenv_path.is_empty() {
    config.dotenv_path.as_slice()
  } else {
    settings.dotenv_path.as_slice()
  };

  if !settings.dotenv_load
    && !settings.dotenv_override
    && !settings.dotenv_required
    && dotenv_filenames.is_empty()
    && dotenv_paths.is_empty()
  {
    return Ok(BTreeMap::new());
  }

  if !dotenv_paths.is_empty() {
    let present_paths = dotenv_paths
      .iter()
      .map(|path| working_directory.join(path))
      .filter(|path| path.is_file())
      .collect::<Vec<_>>();

    if !present_paths.is_empty() {
      return load_from_files(&present_paths, settings);
    }
  }

  let filenames = if dotenv_filenames.is_empty() {
    vec![".env"]
  } else {
    dotenv_filenames
      .iter()
      .map(|s| s.as_str())
      .collect::<Vec<_>>()
  };

  for directory in working_directory.ancestors() {
    let present_filenames = filenames
      .iter()
      .filter_map(|filename| {
        let filename = directory.join(filename);
        if filename.is_file() {
          Some(filename)
        } else {
          None
        }
      })
      .collect::<Vec<_>>();

    if !present_filenames.is_empty() {
      return load_from_files(&present_filenames, settings);
    }
  }

  if settings.dotenv_required {
    Err(Error::DotenvRequired)
  } else {
    Ok(BTreeMap::new())
  }
}

fn load_from_files(
  paths: &[PathBuf],
  settings: &Settings,
) -> RunResult<'static, BTreeMap<String, String>> {
  let mut dotenv = BTreeMap::new();

  for path in paths {
    let iter = dotenvy::from_path_iter(path)?;
    for result in iter {
      let (key, value) = result?;
      if settings.dotenv_override || env::var_os(&key).is_none() {
        dotenv.insert(key, value);
      }
    }
  }

  Ok(dotenv)
}
