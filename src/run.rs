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

  let config = Config::from_matches(&matches).eprint(Color::auto())?;

  let color = config.color;

  match config.run_subcommand() {
    Err(error) => match error {
      JustError::Code(code) => Err(code),
      JustError::Search(error) => Err(error).eprint(color),
    },
    Ok(()) => Ok(()),
  }
}
