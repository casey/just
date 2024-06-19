use super::*;

/// ArgSource controls where the program arguments are received from.
enum ArgSource {
  OS,
  Is(Vec<String>),
}

/// Main entry point into just binary, taking arguments provided by the OS.
pub fn run() -> Result<(), i32> {
  run_with_arg_source(ArgSource::OS)
}

/// Main entry point into just library, taking arguments provided by Vec<&str>
pub fn run_with_args(args: Vec<&str>) -> Result<(), i32> {
  let args: Vec<String> = Vec::from_iter(
    vec!["just".to_string()].into_iter().chain(
      args
        .iter()
        .map(|&s| s.into())
        .collect::<Vec<String>>()
        .iter()
        .cloned(),
    ),
  );
  run_with_arg_source(ArgSource::Is(args))
}

/// Main entry point which takes arguments from a configurable source.
#[allow(clippy::missing_errors_doc)]
fn run_with_arg_source(arg_source: ArgSource) -> Result<(), i32> {
  #[cfg(windows)]
  ansi_term::enable_ansi_support().ok();

  env_logger::Builder::from_env(
    env_logger::Env::new()
      .filter("JUST_LOG")
      .write_style("JUST_LOG_STYLE"),
  )
  .try_init()
  .ok();

  let app = Config::app();

  info!("Parsing command line arguments…");
  let matches = match arg_source {
    ArgSource::OS => app.get_matches(),
    ArgSource::Is(value) => app.get_matches_from(value),
  };

  let config = Config::from_matches(&matches).map_err(Error::from);

  let (color, verbosity) = config
    .as_ref()
    .map(|config| (config.color, config.verbosity))
    .unwrap_or((Color::auto(), Verbosity::default()));

  let loader = Loader::new();

  config
    .and_then(|config| config.run(&loader))
    .map_err(|error| {
      if !verbosity.quiet() && error.print_message() {
        eprintln!("{}", error.color_display(color.stderr()));
      }
      error.code().unwrap_or(EXIT_FAILURE)
    })
}
