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
    if let Some(map) = load_from_file(&path, settings)? {
      return Ok(map);
    }
  }

  let filename = dotenv_filename.map_or(".env", |s| s.as_str());

  for directory in working_directory.ancestors() {
    let path = directory.join(filename);
    if let Some(map) = load_from_file(&path, settings)? {
      return Ok(map);
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
) -> RunResult<'static, Option<BTreeMap<String, String>>> {
  let file = match File::open(path) {
    Ok(file) => file,
    Err(source) => {
      if matches!(
        source.kind(),
        io::ErrorKind::IsADirectory | io::ErrorKind::NotFound,
      ) {
        return Ok(None);
      }
      return Err(Error::FilesystemIo {
        path: path.into(),
        source,
      });
    }
  };

  if file
    .metadata()
    .map_err(|source| Error::FilesystemIo {
      path: path.into(),
      source,
    })?
    .is_dir()
  {
    return Ok(None);
  }

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
