use super::*;

pub(crate) fn load_dotenv(
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  let commands = if config.dotenv_command.is_empty() {
    settings.dotenv_command.elements()
  } else {
    &config.dotenv_command
  };

  if !commands.is_empty() {
    let mut dotenv = BTreeMap::new();
    for command in commands {
      dotenv.extend(load_from_command(
        command,
        config,
        settings,
        working_directory,
      )?);
    }
    return Ok(dotenv);
  }

  if !settings.lists && (config.dotenv_filename.len() > 1 || config.dotenv_path.len() > 1) {
    return Err(Error::DotenvArgumentsRequireLists);
  }

  let filenames = if config.dotenv_filename.is_empty() {
    settings.dotenv_filename.clone()
  } else {
    config.dotenv_filename.clone().into()
  };

  let paths = if config.dotenv_path.is_empty() {
    settings.dotenv_path.clone()
  } else {
    config.dotenv_path.clone().into()
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

  for path in paths {
    let path = working_directory.join(path);
    if let Some(map) = load_from_file(&path, settings)? {
      dotenv.extend(map);
      found = true;
    }
  }

  if found {
    return Ok(dotenv);
  }

  let filenames = if filenames.is_empty() {
    ".env".into()
  } else {
    filenames
  };

  for directory in working_directory.ancestors() {
    for filename in &filenames {
      if let Some(map) = load_from_file(&directory.join(filename), settings)? {
        dotenv.extend(map);
        found = true;
      }
    }

    if found {
      return Ok(dotenv);
    }
  }

  if settings.dotenv_required {
    Err(Error::DotenvRequired)
  } else {
    Ok(BTreeMap::new())
  }
}

fn load_from_command(
  command: &str,
  config: &Config,
  settings: &Settings,
  working_directory: &Path,
) -> RunResult<'static, BTreeMap<String, String>> {
  let mut cmd = settings.shell_command(config);

  cmd
    .arg(command)
    .current_dir(working_directory)
    .stdin(Stdio::inherit())
    .stderr(if config.verbosity.quiet() {
      Stdio::null()
    } else {
      Stdio::inherit()
    })
    .stdout(Stdio::piped());

  let output = cmd
    .output_guard_stdout()
    .map_err(|output_error| Error::DotenvCommand {
      command: command.into(),
      output_error,
    })?;

  let mut dotenv = BTreeMap::new();

  for result in dotenvy::from_read_iter(output.as_bytes()) {
    let (key, value) = result.map_err(|dotenv_error| Error::Dotenv {
      dotenv_error,
      path: working_directory.into(),
    })?;

    if settings.dotenv_override || env::var_os(&key).is_none() {
      dotenv.insert(key, value);
    }
  }

  Ok(dotenv)
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
