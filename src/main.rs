use {
  clap::CommandFactory,
  clap_complete::CompleteEnv,
  just::Arguments,
  std::{env, process},
};

fn main() {
  if env::var_os("JUST_COMPLETE").is_some_and(|value| value == "bash")
    && env::args_os().nth(1).is_none()
  {
    print!("{}", include_str!("../completion-registration-script.bash"));
    return;
  }

  CompleteEnv::with_factory(Arguments::command)
    .var("JUST_COMPLETE")
    .complete();

  if let Err(code) = just::run(env::args_os()) {
    process::exit(code);
  }
}
