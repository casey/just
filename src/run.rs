use crate::common::*;

pub fn run(args: impl Iterator<Item = impl Into<OsString> + Clone>) -> Result<(), i32> {
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
  let matches = app.get_matches_from(args);

  let config = Config::from_matches(&matches).eprint(Color::auto())?;

  config.run_subcommand()
}
