use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Arguments {
  #[structopt(long)]
  reference: String,
}

fn main() {
  let arguments = Arguments::from_args();

  let regex = Regex::new("^refs/tags/[[:digit:]]+[.][[:digit:]]+[.][[:digit:]]+$")
    .expect("Failed to compile release regex");

  println!(
    "::set-output name=value::{}",
    !regex.is_match(&arguments.reference)
  );
}
