fn main() {
  if let Err(code) = just::run(std::env::args_os()) {
    std::process::exit(code);
  }
}
