extern crate clap;
extern crate regex;
extern crate atty;
extern crate ansi_term;

use std::{io, fs, env, process, convert, ffi};
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
    process::exit(-1)
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
    Ok(status) => process::exit(status.code().unwrap_or(-1)),
    Err(error) => die!("Failed to invoke editor: {}", error),
  }
}

pub fn app() {
  let matches = App::new("just")
    .version(concat!("v", env!("CARGO_PKG_VERSION")))
    .author("Casey Rodarmor <casey@rodarmor.com>")
    .about("Just a command runner - https://github.com/casey/just")
    .setting(AppSettings::ColoredHelp)
    .arg(Arg::with_name("arguments")
         .multiple(true)
         .help("The recipe(s) to run, defaults to the first recipe in the justfile"))
    .arg(Arg::with_name("color")
         .long("color")
         .takes_value(true)
         .possible_values(&["auto", "always", "never"])
         .default_value("auto")
         .help("Prints colorful output"))
    .arg(Arg::with_name("dry-run")
         .long("dry-run")
         .help("Prints what just would do without doing it")
         .conflicts_with("quiet"))
    .arg(Arg::with_name("dump")
         .long("dump")
         .help("Prints entire justfile"))
    .arg(Arg::with_name("edit")
         .short("e")
         .long("edit")
         .help("Opens justfile with $EDITOR"))
    .arg(Arg::with_name("evaluate")
         .long("evaluate")
         .help("Prints evaluated variables"))
    .arg(Arg::with_name("justfile")
         .long("justfile")
         .takes_value(true)
         .help("Uses <justfile> as justfile. --working-directory must also be set")
         .requires("working-directory"))
    .arg(Arg::with_name("list")
         .short("l")
         .long("list")
         .help("Lists available recipes and their arguments"))
    .arg(Arg::with_name("quiet")
         .short("q")
         .long("quiet")
         .help("Suppresses all output")
         .conflicts_with("dry-run"))
    .arg(Arg::with_name("set")
         .long("set")
         .takes_value(true)
         .number_of_values(2)
         .value_names(&["variable", "value"])
         .multiple(true)
         .help("Sets <variable> to <value>"))
    .arg(Arg::with_name("shell")
         .long("shell")
         .takes_value(true)
         .default_value(DEFAULT_SHELL)
         .help("Invoke <shell> to run recipes"))
    .arg(Arg::with_name("show")
         .short("s")
         .long("show")
         .takes_value(true)
         .value_name("recipe")
         .help("Shows information about <recipe>"))
    .arg(Arg::with_name("summary")
         .long("summary")
         .help("Lists names of available recipes"))
    .arg(Arg::with_name("verbose")
         .short("v") .long("verbose")
         .help("Use verbose output"))
    .arg(Arg::with_name("working-directory")
        .long("working-directory")
        .takes_value(true)
        .help("Uses <working-directory> as working directory. --justfile must also be set")
        .requires("justfile"))
    .group(ArgGroup::with_name("early-exit")
         .args(&["dump", "edit", "list", "show", "summary", "arguments", "evaluate"]))
    .get_matches();

  let use_color_argument = matches.value_of("color").expect("--color had no value");
  let use_color = match UseColor::from_argument(use_color_argument) {
    Some(use_color) => use_color,
    None => die!("Invalid argument to --color. This is a bug in just."),
  };

  let set_count = matches.occurrences_of("set");
  let mut overrides = BTreeMap::new();
  if set_count > 0 {
    let mut values = matches.values_of("set").unwrap();
    for _ in 0..set_count {
      overrides.insert(values.next().unwrap(), values.next().unwrap());
    }
  }

  let override_re = regex::Regex::new("^([^=]+)=(.*)$").unwrap();

  let raw_arguments = matches.values_of("arguments").map(|values| values.collect::<Vec<_>>())
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
          if matches.is_present("working-directory") {
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

  let justfile_option = matches.value_of("justfile");
  let working_directory_option = matches.value_of("working-directory");

  let text;
  if let (Some(file), Some(directory)) = (justfile_option, working_directory_option) {
    if matches.is_present("edit") {
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

    if matches.is_present("edit") {
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

  if matches.is_present("summary") {
    if justfile.count() == 0 {
      warn!("Justfile contains no recipes.");
    } else {
      println!("{}", justfile.recipes.keys().cloned().collect::<Vec<_>>().join(" "));
    }
    process::exit(0);
  }

  if matches.is_present("dump") {
    println!("{}", justfile);
    process::exit(0);
  }

  if matches.is_present("list") {
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
    process::exit(0);
  }

  if let Some(name) = matches.value_of("show") {
    match justfile.recipes.get(name) {
      Some(recipe) => {
        println!("{}", recipe);
        process::exit(0);
      }
      None => {
        warn!("Justfile does not contain recipe `{}`.", name);
        if let Some(suggestion) = justfile.suggest(name) {
          warn!("Did you mean `{}`?", suggestion);
        }
        process::exit(-1)
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
    dry_run:   matches.is_present("dry-run"),
    evaluate:  matches.is_present("evaluate"),
    overrides: overrides,
    quiet:     matches.is_present("quiet"),
    shell:     matches.value_of("shell"),
    use_color: use_color,
    verbose:   matches.is_present("verbose"),
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
      _ => process::exit(-1),
    }
  }
}
