extern crate j;
extern crate clap;

use std::{io, fs, env};
use clap::{App, Arg};
use j::Slurp;

macro_rules! warn {
  ($($arg:tt)*) => {{
    extern crate std;
    use std::io::prelude::*;
    let _ = writeln!(&mut std::io::stderr(), $($arg)*);
  }};
}
macro_rules! die {
  ($($arg:tt)*) => {{
    extern crate std;
    warn!($($arg)*);
    std::process::exit(-1)
  }};
}

fn main() {
  let matches = App::new("j")
    .version("0.1.5")
    .author("Casey R. <casey@rodarmor.com>")
    .about("Just a command runner")
    .arg(Arg::with_name("list")
         .short("l")
         .long("list")
         .help("Lists available recipes"))
    .arg(Arg::with_name("recipe")
         .multiple(true)
         .help("recipe(s) to run, defaults to the first recipe in the justfile"))
    .get_matches();

  loop {
    match fs::metadata("justfile") {
      Ok(metadata) => if metadata.is_file() { break; },
      Err(error) => {
        if error.kind() != io::ErrorKind::NotFound {
          die!("Error fetching justfile metadata: {}", error)
        }
      }
    }

    match env::current_dir() {
      Ok(pathbuf) => if pathbuf.as_os_str() == "/" { die!("No justfile found."); },
      Err(error) => die!("Error getting current dir: {}", error),
    }

    if let Err(error) = env::set_current_dir("..") {
      die!("Error changing directory: {}", error);
    }
  }

  let text = fs::File::open("justfile")
    .unwrap_or_else(|error| die!("Error opening justfile: {}", error))
    .slurp()
    .unwrap_or_else(|error| die!("Error reading justfile: {}", error));

  let justfile = j::parse(&text).unwrap_or_else(|error| die!("{}", error));

  if matches.is_present("list") {
    if justfile.count() == 0 {
      warn!("Justfile contains no recipes");
    } else {
      warn!("{}", justfile.recipes().join(" "));
    }
    std::process::exit(0);
  }

  let names = if let Some(names) = matches.values_of("recipe") {
    names.collect::<Vec<_>>()
  } else if let Some(name) = justfile.first() {
    vec![name]
  } else {
    die!("Justfile contains no recipes");
  };

  if let Err(run_error) = justfile.run(&names) {
    warn!("{}", run_error);
    std::process::exit(if let j::RunError::Code{code, ..} = run_error { code } else { -1 });
  }
}
