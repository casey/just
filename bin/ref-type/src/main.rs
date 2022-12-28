use {regex::Regex, structopt::StructOpt};

#[derive(StructOpt)]
struct Arguments {
  #[structopt(long)]
  reference: String,
}

fn main() {
  let arguments = Arguments::from_args();

  let regex = Regex::new("^refs/tags/[[:digit:]]+[.][[:digit:]]+[.][[:digit:]]+$")
    .expect("Failed to compile release regex");

  let value = if regex.is_match(&arguments.reference) {
    "release"
  } else {
    "other"
  };

  eprintln!("ref: {}", arguments.reference);
  eprintln!("value: {}", value);

  println!("::set-output name=value::{}", value);
}
