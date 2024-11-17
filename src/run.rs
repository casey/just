use super::*;

/// Main entry point into `just`. Parse arguments from `args` and run.
#[allow(clippy::missing_errors_doc)]
pub fn run(args: impl Iterator<Item = impl Into<OsString> + Clone>) -> Result<(), i32> {
  #[cfg(windows)]
  ansi_term::enable_ansi_support().ok();

  let app = Config::app();

  let matches = app.try_get_matches_from(args).map_err(|err| {
    err.print().ok();
    err.exit_code()
  })?;

  let config = Config::from_matches(&matches).map_err(Error::from);

  let (color, verbosity, allow_missing) = config
    .as_ref()
    .map(|config| (config.color, config.verbosity, config.allow_missing))
    .unwrap_or_default();

  let loader = Loader::new();

  config
    .and_then(|config| {
      InterruptHandler::install(config.verbosity).ok();
      config.subcommand.execute(&config, &loader)
    })
    .map_err(|error| {
      if allow_missing {
        if let Error::UnknownRecipe { .. } = error {
          return 0;
        }
      }

      if !verbosity.quiet() && error.print_message() {
        eprintln!("{}", error.color_display(color.stderr()));
      }
      error.code().unwrap_or(EXIT_FAILURE)
    })
}
