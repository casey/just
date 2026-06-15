use super::*;

pub(crate) fn load_dotenv(
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  let filenames = match &config.dotenv_filename {
    Some(filename) => filename.into(),
    None => settings.dotenv_filename.clone(),
  };

  let paths = match &config.dotenv_path {
    Some(path) => path.into(),
    None => settings.dotenv_path.clone(),
  };

  if !settings.dotenv_load
    && !settings.dotenv_override
    && !settings.dotenv_required
    && filenames.is_empty()
    && paths.is_empty()
  {
    return Ok(BTreeMap::new());
  }

  let mut dotenv = BTreeMap::new();
  let mut found = false;

  for path in paths.elements() {
    let path = working_directory.join(path);
    if let Some(map) = load_from_file(&path, settings)? {
      dotenv.extend(map);
      found = true;
    }
  }

  if !found {
    let default = [".env".to_string()];
    let filenames = if filenames.is_empty() {
      &default[..]
    } else {
      filenames.elements()
    };

    for directory in working_directory.ancestors() {
      let mut matched = Vec::new();
      for filename in filenames {
        if let Some(map) = load_from_file(&directory.join(filename), settings)? {
          matched.push(map);
        }
      }
      if !matched.is_empty() {
        for map in matched {
          dotenv.extend(map);
        }
        found = true;
        break;
      }
    }
  }

  if !found && settings.dotenv_required {
    Err(Error::DotenvRequired)
  } else {
    Ok(dotenv)
  }
}

fn load_from_file(
  path: &Path,
  settings: &Settings,
) -> RunResult<'static, Option<BTreeMap<String, String>>> {
  if path.is_dir() {
    return Ok(None);
  }

  let file = match File::open(path) {
    Ok(file) => file,
    Err(source) => {
      if source.kind() == io::ErrorKind::NotFound {
        return Ok(None);
      }
      return Err(Error::FilesystemIo {
        path: path.into(),
        source,
      });
    }
  };

  let mut dotenv = BTreeMap::new();

  for result in dotenvy::from_read_iter(file) {
    let (key, value) = result.map_err(|dotenv_error| Error::Dotenv {
      dotenv_error,
      path: path.into(),
    })?;

    if settings.dotenv_override || env::var_os(&key).is_none() {
      dotenv.insert(key, value);
    }
  }

  Ok(Some(dotenv))
}
