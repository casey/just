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
  let verbosity = config.verbosity;

  let loader = Loader::new();

  match config.run_subcommand(&loader) {
    Err(error) => match error {
      JustError::Code(code) => Err(code),
      JustError::Search(error) => Err(error).eprint(color),
      JustError::Load(error) => Err(error).eprint(color),
      JustError::Compile(error) => Err(error).eprint(color),
      JustError::Run(error) =>
        if !verbosity.quiet() {
          Err(error).eprint(color)
        } else {
          Err(error.code())
        },
    },
    Ok(()) => Ok(()),
  }
}
