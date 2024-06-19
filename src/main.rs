fn main() {
  if let Err(code) = just::run(std::env::args_os().into_iter().collect()) {
    std::process::exit(code);
  }
}
