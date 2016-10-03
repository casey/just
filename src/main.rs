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

  if let Some(recipes) = matches.values_of("recipe") {
    let mut missing = vec![];
    for recipe in recipes {
      if !justfile.recipes.contains_key(recipe) {
        missing.push(recipe);
      }
    }
    if missing.len() > 0 {
      die!("unknown recipe{}: {}", if missing.len() == 1 { "" } else { "s" }, missing.join(" "));
    }
  }

  if matches.is_present("list") {
    if justfile.recipes.len() == 0 {
      warn!("Justfile contains no recipes");
    } else {
      warn!("{}", justfile.recipes.keys().cloned().collect::<Vec<_>>().join(" "));
    }
    std::process::exit(0);
  }

  if let Some(values) = matches.values_of("recipe") {
    let names = values.collect::<Vec<_>>();
    for name in names.iter() {
      if !justfile.contains(name) {
        die!("Justfile does not contain recipe \"{}\"", name);
      }
    }
    justfile.run(&names)
  } else if let Some(name) = justfile.first() {
    justfile.run(&[name])
  } else {
    die!("Justfile contains no recipes");
  }
}
