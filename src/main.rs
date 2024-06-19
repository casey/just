fn main() {
  if let Err(code) = just::run(std::env::args_os().collect()) {
    std::process::exit(code);
  }
}
