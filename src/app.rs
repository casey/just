extern crate clap;
extern crate regex;
extern crate atty;

use std::{io, fs, env, process};
use std::collections::BTreeMap;
use self::clap::{App, Arg, AppSettings};
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

#[derive(Copy, Clone)]
enum UseColor {
  Auto,
  Always,
  Never,
}

impl UseColor {
  fn from_argument(use_color: &str) -> UseColor {
    match use_color {
      "auto"   => UseColor::Auto,
      "always" => UseColor::Always,
      "never"  => UseColor::Never,
      _        => panic!("Invalid argument to --color. This is a bug in just."),
    }
  }

  fn should_color_stream(self, stream: atty::Stream) -> bool {
    match self {
      UseColor::Auto   => atty::is(stream),
      UseColor::Always => true,
      UseColor::Never  => false,
    }
  }
}

pub fn app() {
  let matches = App::new("just")
    .version(concat!("v", env!("CARGO_PKG_VERSION")))
    .author("Casey Rodarmor <casey@rodarmor.com>")
    .about("Just a command runner - https://github.com/casey/just")
    .setting(AppSettings::ColoredHelp)
    .arg(Arg::with_name("list")
         .short("l")
         .long("list")
         .help("Lists available recipes and their arguments")
         .conflicts_with("dump")
         .conflicts_with("show")
         .conflicts_with("summary"))
    .arg(Arg::with_name("dump")
         .long("dump")
         .help("Prints entire justfile")
         .conflicts_with("show")
         .conflicts_with("summary")
         .conflicts_with("list"))
    .arg(Arg::with_name("show")
         .short("s")
         .long("show")
         .takes_value(true)
         .value_name("recipe")
         .help("Shows information about <recipe>")
         .conflicts_with("dump")
         .conflicts_with("summary")
         .conflicts_with("list"))
    .arg(Arg::with_name("summary")
         .long("summary")
         .help("Lists names of available recipes")
         .conflicts_with("dump")
         .conflicts_with("show")
         .conflicts_with("list"))
    .arg(Arg::with_name("quiet")
         .short("q")
         .long("quiet")
         .help("Suppresses all output")
         .conflicts_with("dry-run"))
    .arg(Arg::with_name("dry-run")
         .long("dry-run")
         .help("Prints what just would do without doing it")
         .conflicts_with("quiet"))
    .arg(Arg::with_name("evaluate")
         .long("evaluate")
         .help("Prints evaluated variables"))
    .arg(Arg::with_name("color")
         .long("color")
         .takes_value(true)
         .possible_values(&["auto", "always", "never"])
         .default_value("auto")
         .help("Prints colorful output"))
    .arg(Arg::with_name("set")
         .long("set")
         .takes_value(true)
         .number_of_values(2)
         .value_names(&["variable", "value"])
         .multiple(true)
         .help("Sets <variable> to <value>"))
    .arg(Arg::with_name("working-directory")
         .long("working-directory")
         .takes_value(true)
         .help("Uses <working-directory> as working directory. --justfile must also be set")
         .requires("justfile"))
    .arg(Arg::with_name("justfile")
         .long("justfile")
         .takes_value(true)
         .help("Uses <justfile> as justfile. --working-directory must also be set")
         .requires("working-directory"))
    .arg(Arg::with_name("arguments")
         .multiple(true)
         .help("The recipe(s) to run, defaults to the first recipe in the justfile"))
    .get_matches();

  let use_color_argument = matches.value_of("color").expect("--color had no value");
  let use_color = UseColor::from_argument(use_color_argument);

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
    let name;
    'outer: loop {
      for candidate in &["justfile", "Justfile"] {
        match fs::metadata(candidate) {
          Ok(metadata) => if metadata.is_file() {
            name = *candidate;
            break 'outer;
          },
          Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
              die!("Error fetching justfile metadata: {}", error)
            }
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

    text = fs::File::open(name)
      .unwrap_or_else(|error| die!("Error opening justfile: {}", error))
      .slurp()
      .unwrap_or_else(|error| die!("Error reading justfile: {}", error));
  }

  let justfile = super::parse(&text).unwrap_or_else(|error|
    if use_color.should_color_stream(atty::Stream::Stderr) {
      die!("{:#}", error);
    } else {
      die!("{}", error);
    }
  );

  if matches.is_present("summary") {
    if justfile.count() == 0 {
      warn!("Justfile contains no recipes.");
    } else {
      println!("{}", justfile.recipes().join(" "));
    }
    process::exit(0);
  }

  if matches.is_present("dump") {
    println!("{}", justfile);
    process::exit(0);
  }

  if matches.is_present("list") {
    println!("Available recipes:");
    for (name, recipe) in &justfile.recipes {
      print!("    {}", name);
      for parameter in &recipe.parameters {
        print!(" {}", parameter);
      }
      println!("");
    }
    process::exit(0);
  }

  if let Some(name) = matches.value_of("show") {
    match justfile.recipes.get(name) {
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
      if use_color.should_color_stream(atty::Stream::Stderr) {
        warn!("{:#}", run_error);
      } else {
        warn!("{}", run_error);
      }
    }
    match run_error {
      RunError::Code{code, .. } | RunError::BacktickCode{code, ..} => process::exit(code),
      _ => process::exit(-1),
    }
  }
}
