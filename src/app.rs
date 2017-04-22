extern crate ansi_term;
extern crate atty;
extern crate clap;
extern crate libc;
extern crate regex;

use ::prelude::*;
use std::{convert, ffi};
use std::collections::BTreeMap;
use self::clap::{App, Arg, ArgGroup, AppSettings};
use super::{Slurp, RunError, RunOptions, compile, DEFAULT_SHELL};

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
    process::exit(EXIT_FAILURE)
  }};
}

#[derive(Copy, Clone)]
pub enum UseColor {
  Auto,
  Always,
  Never,
}

impl Default for UseColor {
  fn default() -> UseColor {
    UseColor::Never
  }
}

impl UseColor {
  fn from_argument(use_color: &str) -> Option<UseColor> {
    match use_color {
      "auto"   => Some(UseColor::Auto),
      "always" => Some(UseColor::Always),
      "never"  => Some(UseColor::Never),
      _        => None,
    }
  }

  fn should_color_stream(self, stream: atty::Stream) -> bool {
    match self {
      UseColor::Auto   => atty::is(stream),
      UseColor::Always => true,
      UseColor::Never  => false,
    }
  }

  pub fn should_color_stdout(self) -> bool {
    self.should_color_stream(atty::Stream::Stdout)
  }

  pub fn should_color_stderr(self) -> bool {
    self.should_color_stream(atty::Stream::Stderr)
  }

  fn blue(self, stream: atty::Stream) -> ansi_term::Style {
    if self.should_color_stream(stream) {
      ansi_term::Style::new().fg(ansi_term::Color::Blue)
    } else {
      ansi_term::Style::default()
    }
  }
}

fn edit<P: convert::AsRef<ffi::OsStr>>(path: P) -> ! {
  let editor = env::var_os("EDITOR")
    .unwrap_or_else(|| die!("Error getting EDITOR environment variable"));

  let error = process::Command::new(editor)
    .arg(path)
    .status();

  match error {
    Ok(status) => process::exit(status.code().unwrap_or(EXIT_FAILURE)),
    Err(error) => die!("Failed to invoke editor: {}", error),
  }
}

pub fn app() {
  let matches = App::new("just")
    .version(concat!("v", env!("CARGO_PKG_VERSION")))
    .author("Casey Rodarmor <casey@rodarmor.com>")
    .about("Just a command runner - https://github.com/casey/just")
    .setting(AppSettings::ColoredHelp)
    .arg(Arg::with_name("ARGUMENTS")
         .multiple(true)
         .help("The recipe(s) to run, defaults to the first recipe in the justfile"))
    .arg(Arg::with_name("COLOR")
         .long("color")
         .takes_value(true)
         .possible_values(&["auto", "always", "never"])
         .default_value("auto")
         .help("Prints colorful output"))
    .arg(Arg::with_name("DRY-RUN")
         .long("dry-run")
         .help("Prints what just would do without doing it")
         .conflicts_with("quiet"))
    .arg(Arg::with_name("DUMP")
         .long("dump")
         .help("Prints entire justfile"))
    .arg(Arg::with_name("EDIT")
         .short("e")
         .long("edit")
         .help("Opens justfile with $EDITOR"))
    .arg(Arg::with_name("EVALUATE")
         .long("evaluate")
         .help("Prints evaluated variables"))
    .arg(Arg::with_name("JUSTFILE")
         .long("justfile")
         .takes_value(true)
         .help("Uses <JUSTFILE> as justfile. --working-directory must also be set")
         .requires("WORKING-DIRECTORY"))
    .arg(Arg::with_name("LIST")
         .short("l")
         .long("list")
         .help("Lists available recipes and their arguments"))
    .arg(Arg::with_name("QUIET")
         .short("q")
         .long("quiet")
         .help("Suppresses all output")
         .conflicts_with("DRY-RUN"))
    .arg(Arg::with_name("SET")
         .long("set")
         .takes_value(true)
         .number_of_values(2)
         .value_names(&["VARIABLE", "VALUE"])
         .multiple(true)
         .help("Sets <VARIABLE> to <VALUE>"))
    .arg(Arg::with_name("SHELL")
         .long("shell")
         .takes_value(true)
         .default_value(DEFAULT_SHELL)
         .help("Invoke <SHELL> to run recipes"))
    .arg(Arg::with_name("SHOW")
         .short("s")
         .long("show")
         .takes_value(true)
         .value_name("RECIPE")
         .help("Shows information about <RECIPE>"))
    .arg(Arg::with_name("SUMMARY")
         .long("summary")
         .help("Lists names of available recipes"))
    .arg(Arg::with_name("VERBOSE")
         .short("v")
         .long("verbose")
         .help("Use verbose output"))
    .arg(Arg::with_name("WORKING-DIRECTORY")
        .long("working-directory")
        .takes_value(true)
        .help("Uses <WORKING-DIRECTORY> as working directory. --justfile must also be set")
        .requires("JUSTFILE"))
    .group(ArgGroup::with_name("EARLY-EXIT")
         .args(&["DUMP", "EDIT", "LIST", "SHOW", "SUMMARY", "ARGUMENTS", "EVALUATE"]))
    .get_matches();

  let use_color_argument = matches.value_of("COLOR").expect("--color had no value");
  let use_color = match UseColor::from_argument(use_color_argument) {
    Some(use_color) => use_color,
    None => die!("Invalid argument to --color. This is a bug in just."),
  };

  let set_count = matches.occurrences_of("SET");
  let mut overrides = BTreeMap::new();
  if set_count > 0 {
    let mut values = matches.values_of("SET").unwrap();
    for _ in 0..set_count {
      overrides.insert(values.next().unwrap(), values.next().unwrap());
    }
  }

  let override_re = regex::Regex::new("^([^=]+)=(.*)$").unwrap();

  let raw_arguments = matches.values_of("ARGUMENTS").map(|values| values.collect::<Vec<_>>())
    .unwrap_or_default();

  for argument in raw_arguments.iter().take_while(|arg| override_re.is_match(arg)) {
    let captures = override_re.captures(argument).unwrap();
    overrides.insert(captures.at(1).unwrap(), captures.at(2).unwrap());
  }

  let rest = raw_arguments.iter().skip_while(|arg| override_re.is_match(arg))
    .enumerate()
    .flat_map(|(i, argument)| {
      if i == 0 {
        if let Some(i) = argument.rfind('/') {
          if matches.is_present("WORKING-DIRECTORY") {
            die!("--working-directory and a path prefixed recipe may not be used together.");
          }

          let (dir, recipe) = argument.split_at(i + 1);

          if let Err(error) = env::set_current_dir(dir) {
            die!("Error changing directory: {}", error);
          }

          if recipe.is_empty() {
            return None;
          } else {
            return Some(recipe);
          }
        }
      }

      Some(*argument)
    })
    .collect::<Vec<&str>>();

  let justfile_option = matches.value_of("JUSTFILE");
  let working_directory_option = matches.value_of("WORKING-DIRECTORY");

  let text;
  if let (Some(file), Some(directory)) = (justfile_option, working_directory_option) {
    if matches.is_present("EDIT") {
      edit(file);
    }

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

    if matches.is_present("EDIT") {
      edit(name);
    }

    text = fs::File::open(name)
      .unwrap_or_else(|error| die!("Error opening justfile: {}", error))
      .slurp()
      .unwrap_or_else(|error| die!("Error reading justfile: {}", error));
  }

  let justfile = compile(&text).unwrap_or_else(|error|
    if use_color.should_color_stderr() {
      die!("{:#}", error);
    } else {
      die!("{}", error);
    }
  );

  if matches.is_present("SUMMARY") {
    if justfile.count() == 0 {
      warn!("Justfile contains no recipes.");
    } else {
      println!("{}", justfile.recipes.keys().cloned().collect::<Vec<_>>().join(" "));
    }
    process::exit(EXIT_SUCCESS);
  }

  if matches.is_present("DUMP") {
    println!("{}", justfile);
    process::exit(EXIT_SUCCESS);
  }

  if matches.is_present("LIST") {
    let blue = use_color.blue(atty::Stream::Stdout);
    println!("Available recipes:");
    for (name, recipe) in &justfile.recipes {
      print!("    {}", name);
      for parameter in &recipe.parameters {
        if use_color.should_color_stdout() {
          print!(" {:#}", parameter);
        } else {
          print!(" {}", parameter);
        }
      }
      if let Some(doc) = recipe.doc {
        print!(" {} {}", blue.paint("#"), blue.paint(doc));
      }
      println!("");
    }
    process::exit(EXIT_SUCCESS);
  }

  if let Some(name) = matches.value_of("SHOW") {
    match justfile.recipes.get(name) {
      Some(recipe) => {
        println!("{}", recipe);
        process::exit(EXIT_SUCCESS);
      }
      None => {
        warn!("Justfile does not contain recipe `{}`.", name);
        if let Some(suggestion) = justfile.suggest(name) {
          warn!("Did you mean `{}`?", suggestion);
        }
        process::exit(EXIT_FAILURE)
      }
    }
  }

  let arguments = if !rest.is_empty() {
    rest
  } else if let Some(recipe) = justfile.first() {
    vec![recipe]
  } else {
    die!("Justfile contains no recipes");
  };

  let options = RunOptions {
    dry_run:   matches.is_present("DRY-RUN"),
    evaluate:  matches.is_present("EVALUATE"),
    overrides: overrides,
    quiet:     matches.is_present("QUIET"),
    shell:     matches.value_of("SHELL"),
    use_color: use_color,
    verbose:   matches.is_present("VERBOSE"),
  };

  if let Err(run_error) = justfile.run(&arguments, &options) {
    if !options.quiet {
      if use_color.should_color_stderr() {
        warn!("{:#}", run_error);
      } else {
        warn!("{}", run_error);
      }
    }
    match run_error {
      RunError::Code{code, .. } | RunError::BacktickCode{code, ..} => process::exit(code),
      _ => process::exit(libc::EXIT_FAILURE),
    }
  }
}
