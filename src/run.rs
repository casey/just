use crate::common::*;

pub fn run() -> Result<(), i32> {
  #[cfg(windows)]
  ansi_term::enable_ansi_support().ok();

  env_logger::Builder::from_env(
    env_logger::Env::new()
      .filter("JUST_LOG")
      .write_style("JUST_LOG_STYLE"),
  )
  .init();

  let app = Config::app();

  let matches = app.get_matches();

  let config = match Config::from_matches(&matches) {
    Ok(config) => config,
    Err(error) => {
      eprintln!("error: {}", error);
      return Err(EXIT_FAILURE);
    }
  };

  let justfile = config.justfile;

  if let Some(directory) = config.search_directory {
    if let Err(error) = env::set_current_dir(&directory) {
      die!(
        "Error changing directory to {}: {}",
        directory.display(),
        error
      );
    }
  }

  let mut working_directory = config.working_directory.map(PathBuf::from);

  if let (Some(justfile), None) = (justfile, working_directory.as_ref()) {
    let mut justfile = justfile.to_path_buf();

    if !justfile.is_absolute() {
      match justfile.canonicalize() {
        Ok(canonical) => justfile = canonical,
        Err(err) => {
          eprintln!(
            "Could not canonicalize justfile path `{}`: {}",
            justfile.display(),
            err
          );
          return Err(EXIT_FAILURE);
        }
      }
    }

    justfile.pop();

    working_directory = Some(justfile);
  }

  let text;
  if let (Some(justfile), Some(directory)) = (justfile, working_directory) {
    if config.subcommand == Subcommand::Edit {
      return Subcommand::edit(justfile);
    }

    text = fs::read_to_string(justfile)
      .unwrap_or_else(|error| die!("Error reading justfile: {}", error));

    if let Err(error) = env::set_current_dir(&directory) {
      die!(
        "Error changing directory to {}: {}",
        directory.display(),
        error
      );
    }
  } else {
    let current_dir = match env::current_dir() {
      Ok(current_dir) => current_dir,
      Err(io_error) => die!("Error getting current dir: {}", io_error),
    };
    match search::justfile(&current_dir) {
      Ok(name) => {
        if config.subcommand == Subcommand::Edit {
          return Subcommand::edit(&name);
        }
        text = match fs::read_to_string(&name) {
          Err(error) => {
            eprintln!("Error reading justfile: {}", error);
            return Err(EXIT_FAILURE);
          }
          Ok(text) => text,
        };

        let parent = name.parent().unwrap();

        if let Err(error) = env::set_current_dir(&parent) {
          eprintln!(
            "Error changing directory to {}: {}",
            parent.display(),
            error
          );
          return Err(EXIT_FAILURE);
        }
      }
      Err(search_error) => {
        eprintln!("{}", search_error);
        return Err(EXIT_FAILURE);
      }
    }
  }

  let justfile = match Compiler::compile(&text) {
    Err(error) => {
      if config.color.stderr().active() {
        eprintln!("{:#}", error);
      } else {
        eprintln!("{}", error);
      }
      return Err(EXIT_FAILURE);
    }
    Ok(justfile) => justfile,
  };

  for warning in &justfile.warnings {
    if config.color.stderr().active() {
      eprintln!("{:#}", warning);
    } else {
      eprintln!("{}", warning);
    }
  }

  config.subcommand.run(&config, justfile)
}
