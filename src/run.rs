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

  info!("Parsing command line arguments…");
  let matches = app.get_matches();

  let loader = Loader::new();

  let mut color = Color::auto();
  let mut verbosity = Verbosity::default();

  Config::from_matches(&matches)
    .map_err(Error::from)
    .and_then(|config| {
      color = config.color;
      verbosity = config.verbosity;
      config.run(&loader)
    })
    .map_err(|error| {
      if !verbosity.quiet() {
        eprintln!("{}", error.color_display(color.stderr()));
      }
      error.code().unwrap_or(EXIT_FAILURE)
    })
}
