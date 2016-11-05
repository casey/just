extern crate clap;
extern crate regex;

use std::{io, fs, env, process};
use std::collections::BTreeMap;
use self::clap::{App, Arg};
use super::{Slurp, RunError};

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
  let matches = App::new("just")
    .version("0.2.15")
    .author("Casey Rodarmor <casey@rodarmor.com>")
    .about("Just a command runner - https://github.com/casey/just")
    .arg(Arg::with_name("list")
         .short("l")
         .long("list")
         .help("Lists available recipes"))
    .arg(Arg::with_name("quiet")
         .short("q")
         .long("quiet")
         .help("Suppress all output"))
    .arg(Arg::with_name("dry-run")
         .long("dry-run")
         .help("Print recipe text without executing"))
    .arg(Arg::with_name("evaluate")
         .long("evaluate")
         .help("Print evaluated variables"))
    .arg(Arg::with_name("show")
         .short("s")
         .long("show")
         .takes_value(true)
         .value_name("recipe")
         .help("Show information about <recipe>"))
    .arg(Arg::with_name("set")
         .long("set")
         .takes_value(true)
         .number_of_values(2)
         .value_names(&["variable", "value"])
         .multiple(true)
         .help("set <variable> to <value>"))
    .arg(Arg::with_name("working-directory")
         .long("working-directory")
         .takes_value(true)
         .help("Use <working-directory> as working directory. --justfile must also be set"))
    .arg(Arg::with_name("justfile")
         .long("justfile")
         .takes_value(true)
         .help("Use <justfile> as justfile. --working-directory must also be set"))
    .arg(Arg::with_name("arguments")
         .multiple(true)
         .help("recipe(s) to run, defaults to the first recipe in the justfile"))
    .get_matches();

  // it is not obvious to me what we should do if only one of --justfile and
  // --working-directory are passed. refuse to run in that case to avoid
  // suprises.
  if matches.is_present("justfile") ^ matches.is_present("working-directory") {
    die!("--justfile and --working-directory may only be used together");
  }

  // --dry-run and --quiet don't make sense together
  if matches.is_present("dry-run") && matches.is_present("quiet") {
    die!("--dry-run and --quiet may not be used together");
  }

  let justfile_option = matches.value_of("justfile");
  let working_directory_option = matches.value_of("working-directory");

  let text;
  if let (Some(file), Some(directory)) = (justfile_option, working_directory_option) {
    text = fs::File::open(file)
      .unwrap_or_else(|error| die!("Error opening justfile: {}", error))
      .slurp()
      .unwrap_or_else(|error| die!("Error reading justfile: {}", error));

    if let Err(error) = env::set_current_dir(directory) {
      die!("Error changing directory to {}: {}", directory, error);
    }
  } else {
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

    text = fs::File::open("justfile")
      .unwrap_or_else(|error| die!("Error opening justfile: {}", error))
      .slurp()
      .unwrap_or_else(|error| die!("Error reading justfile: {}", error));
  }

  let justfile = super::parse(&text).unwrap_or_else(|error| die!("{}", error));

  if matches.is_present("list") {
    if justfile.count() == 0 {
      warn!("Justfile contains no recipes");
    } else {
      println!("{}", justfile.recipes().join(" "));
    }
    process::exit(0);
  }

  if let Some(name) = matches.value_of("show") {
    match justfile.get(name) {
      Some(recipe) => {
        println!("{}", recipe);
        process::exit(0);
      }
      None => die!("justfile contains no recipe \"{}\"", name)
    }
  }

  let set_count = matches.occurrences_of("set");
  let mut overrides = BTreeMap::new();
  if set_count > 0 {
    let mut values = matches.values_of("set").unwrap();
    for _ in 0..set_count {
      overrides.insert(values.next().unwrap(), values.next().unwrap());
    }
  }

  let override_re = regex::Regex::new("^([^=]+)=(.*)$").unwrap();

  let arguments = if let Some(arguments) = matches.values_of("arguments") {
    let mut done = false;
    let mut rest = vec![];
    for argument in arguments {
      if !done && override_re.is_match(argument) {
        let captures = override_re.captures(argument).unwrap();
        overrides.insert(captures.at(1).unwrap(), captures.at(2).unwrap());
      } else {
        rest.push(argument);
        done = true;
      }
    }
    rest
  } else if let Some(recipe) = justfile.first() {
    vec![recipe]
  } else {
    die!("Justfile contains no recipes");
  };

  let options = super::RunOptions {
    dry_run:   matches.is_present("dry-run"),
    evaluate:  matches.is_present("evaluate"),
    overrides: overrides,
    quiet:     matches.is_present("quiet"),
  };

  if let Err(run_error) = justfile.run(&arguments, &options) {
    if !options.quiet {
      warn!("{}", run_error);
    }
    match run_error {
      RunError::Code{code, .. } | RunError::BacktickCode{code, ..} => process::exit(code),
      _ => process::exit(-1),
    }
  }
}
