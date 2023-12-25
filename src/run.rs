use super::*;

/// Main entry point into just binary.
#[allow(clippy::missing_errors_doc)]
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

  info!("Parsing command line arguments…");
  let matches = app.get_matches();

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
      match error.code() {
        Some(code) => code,
        None => match error {
          Error::Signal { signal, .. }
          | Error::Backtick {
            output_error: OutputError::Signal(signal),
            ..
          } => Platform::exit_code_from_signal(signal),
          Error::CommandStatus { status, .. } => match status.code() {
            Some(code) => code,
            None => Platform::signal_from_exit_status(status)
              .map(Platform::exit_code_from_signal)
              .unwrap_or(EXIT_FAILURE),
          },
          _ => EXIT_FAILURE,
        },
      }
    })
}
