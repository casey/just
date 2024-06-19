// Example showing how to run just commands from Rust code:

fn main() {
  let args = vec!["-f", "kitchen-sink.just", "foo"];
  if let Err(code) = just::run_with_args(args) {
    std::process::exit(code);
  }
}
