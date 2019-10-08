fn main() {
  if let Err(code) = just::run() {
    std::process::exit(code);
  }
}
