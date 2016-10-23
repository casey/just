extern crate clap;

use std::{io, fs, env, process};
use self::clap::{App, Arg};
use super::Slurp;

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
    process::exit(-1)
  }};
}

pub fn app() {
  let matches = App::new("j")
    .version("0.2.0")
    .author("Casey R. <casey@rodarmor.com>")
    .about("Just a command runner")
    .arg(Arg::with_name("list")
         .short("l")
         .long("list")
         .help("Lists available recipes"))
    .arg(Arg::with_name("show")
         .short("s")
         .long("show")
         .takes_value(true)
         .help("Show information about a recipe"))
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

  let justfile = super::parse(&text).unwrap_or_else(|error| die!("{}", error));

  if matches.is_present("list") {
    if justfile.count() == 0 {
      warn!("Justfile contains no recipes");
    } else {
      warn!("{}", justfile.recipes().join(" "));
    }
    process::exit(0);
  }

  if let Some(name) = matches.value_of("show") {
    match justfile.get(name) {
      Some(recipe) => {
        warn!("{}", recipe);
        process::exit(0);
      }
      None => die!("justfile contains no recipe \"{}\"", name)
    }
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
    //process::exit(if let super::RunError::Code{code, ..} = run_error { code } else { -1 });
    process::exit(-1);
  }
}
