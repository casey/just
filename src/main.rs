use {
  clap::CommandFactory,
  clap_complete::CompleteEnv,
  just::Arguments,
  std::{env, process},
};

fn main() {
  CompleteEnv::with_factory(Arguments::command)
    .var(Arguments::COMPLETION_ENVIRONMENT_VARIABLE)
    .complete();

  if let Err(code) = just::run(env::args_os()) {
    process::exit(code);
  }
}
