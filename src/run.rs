use common::*;

use std::{convert, ffi};
use clap::{App, Arg, ArgGroup, AppSettings};
use configuration::DEFAULT_SHELL;
use misc::maybe_s;
use unicode_width::UnicodeWidthStr;
use env_logger;
use interrupt_handler::InterruptHandler;

#[cfg(windows)]
use ansi_term::enable_ansi_support;

fn edit<P: convert::AsRef<ffi::OsStr>>(path: P) -> ! {
  let editor = env::var_os("EDITOR")
    .unwrap_or_else(|| die!("Error getting EDITOR environment variable"));

  let error = Command::new(editor)
    .arg(path)
    .status();

  match error {
    Ok(status) => process::exit(status.code().unwrap_or(EXIT_FAILURE)),
    Err(error) => die!("Failed to invoke editor: {}", error),
  }
}

trait Slurp {
  fn slurp(&mut self) -> Result<String, io::Error>;
}

impl Slurp for fs::File {
  fn slurp(&mut self) -> io::Result<String> {
    let mut destination = String::new();
    self.read_to_string(&mut destination)?;
    Ok(destination)
  }
}

pub fn run() {
  #[cfg(windows)]
  enable_ansi_support().ok();

  env_logger::Builder::from_env(
    env_logger::Env::new().filter("JUST_LOG").write_style("JUST_LOG_STYLE")
  ).init();

  let invocation_directory = env::current_dir()
    .map_err(|e| format!("Error getting current directory: {}", e));

  let matches = App::new(env!("CARGO_PKG_NAME"))
    .version(concat!("v", env!("CARGO_PKG_VERSION")))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(concat!(env!("CARGO_PKG_DESCRIPTION"), " - ", env!("CARGO_PKG_HOMEPAGE")))
    .help_message("Print help information")
    .version_message("Print version information")
    .setting(AppSettings::ColoredHelp)
    .setting(AppSettings::TrailingVarArg)
    .arg(Arg::with_name("ARGUMENTS")
         .multiple(true)
         .help("The recipe(s) to run, defaults to the first recipe in the justfile"))
    .arg(Arg::with_name("COLOR")
         .long("color")
         .takes_value(true)
         .possible_values(&["auto", "always", "never"])
         .default_value("auto")
         .help("Print colorful output"))
    .arg(Arg::with_name("DRY-RUN")
         .long("dry-run")
         .help("Print what just would do without doing it")
         .conflicts_with("QUIET"))
    .arg(Arg::with_name("DUMP")
         .long("dump")
         .help("Print entire justfile"))
    .arg(Arg::with_name("EDIT")
         .short("e")
         .long("edit")
         .help("Open justfile with $EDITOR"))
    .arg(Arg::with_name("EVALUATE")
         .long("evaluate")
         .help("Print evaluated variables"))
    .arg(Arg::with_name("HIGHLIGHT")
         .long("highlight")
         .help("Highlight echoed recipe lines in bold"))
    .arg(Arg::with_name("JUSTFILE")
         .short("f")
         .long("justfile")
         .takes_value(true)
         .help("Use <JUSTFILE> as justfile"))
    .arg(Arg::with_name("LIST")
         .short("l")
         .long("list")
         .help("List available recipes and their arguments"))
    .arg(Arg::with_name("QUIET")
         .short("q")
         .long("quiet")
         .help("Suppress all output")
         .conflicts_with("DRY-RUN"))
    .arg(Arg::with_name("SET")
         .long("set")
         .takes_value(true)
         .number_of_values(2)
         .value_names(&["VARIABLE", "VALUE"])
         .multiple(true)
         .help("Set <VARIABLE> to <VALUE>"))
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
         .help("Show information about <RECIPE>"))
    .arg(Arg::with_name("SUMMARY")
         .long("summary")
         .help("List names of available recipes"))
    .arg(Arg::with_name("VERBOSE")
         .short("v")
         .long("verbose")
         .multiple(true)
         .help("Use verbose output"))
    .arg(Arg::with_name("WORKING-DIRECTORY")
        .short("d")
        .long("working-directory")
        .takes_value(true)
        .help("Use <WORKING-DIRECTORY> as working directory. --justfile must also be set")
        .requires("JUSTFILE"))
    .group(ArgGroup::with_name("EARLY-EXIT")
         .args(&["DUMP", "EDIT", "LIST", "SHOW", "SUMMARY", "ARGUMENTS", "EVALUATE"]))
    .get_matches();

  let color = match matches.value_of("COLOR").expect("`--color` had no value") {
    "auto"   => Color::auto(),
    "always" => Color::always(),
    "never"  => Color::never(),
    other    => die!("Invalid argument `{}` to --color. This is a bug in just.", other),
  };

  let set_count = matches.occurrences_of("SET");
  let mut overrides = Map::new();
  if set_count > 0 {
    let mut values = matches.values_of("SET").unwrap();
    for _ in 0..set_count {
      overrides.insert(values.next().unwrap(), values.next().unwrap());
    }
  }

  let override_re = Regex::new("^([^=]+)=(.*)$").unwrap();

  let raw_arguments = matches.values_of("ARGUMENTS").map(|values| values.collect::<Vec<_>>())
    .unwrap_or_default();

  for argument in raw_arguments.iter().take_while(|arg| override_re.is_match(arg)) {
    let captures = override_re.captures(argument).unwrap();
    overrides.insert(captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str());
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
  let mut working_directory_option = matches.value_of("WORKING-DIRECTORY");
  
  if let (Some(justfile), None) = (justfile_option, working_directory_option) {
      let justfile_path = Path::new(justfile);
    if Path::new(justfile).is_absolute() {
        working_directory_option = Path::new(justfile).parent().unwrap().to_str();
    } else {
      let justfile_path_canonical = justfile_path.canonicalize();
      if justfile_path.canonicalize().is_ok() {
        working_directory_option = justfile_path_canonical.ok().unwrap().to_str();   
      } else {
        die!("Could not find parent directory of justfile at {}.", justfile);
      }
    }
  }
  
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

  let justfile = Parser::parse(&text).unwrap_or_else(|error|
    if color.stderr().active() {
      die!("{:#}", error);
    } else {
      die!("{}", error);
    }
  );

  if matches.is_present("SUMMARY") {
    if justfile.count() == 0 {
      eprintln!("Justfile contains no recipes.");
    } else {
      let summary = justfile.recipes.iter()
        .filter(|&(_, recipe)| !recipe.private)
        .map(|(name, _)| name)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
      println!("{}", summary);
    }
    process::exit(EXIT_SUCCESS);
  }

  if matches.is_present("DUMP") {
    println!("{}", justfile);
    process::exit(EXIT_SUCCESS);
  }

  if matches.is_present("LIST") {
    let mut line_widths: Map<&str, usize> = Map::new();

    for (name, recipe) in &justfile.recipes {
      if recipe.private {
        continue;
      }

      let mut line_width = UnicodeWidthStr::width(*name);

      for parameter in &recipe.parameters {
        line_width += UnicodeWidthStr::width(format!(" {}", parameter).as_str());
      }

      if line_width <= 30 {
        line_widths.insert(name, line_width);
      }
    }

    let max_line_width = cmp::min(line_widths.values().cloned().max().unwrap_or(0), 30);

    let doc_color = color.stdout().doc();
    println!("Available recipes:");
    for (name, recipe) in &justfile.recipes {
      if recipe.private {
        continue;
      }
      print!("    {}", name);
      for parameter in &recipe.parameters {
        if color.stdout().active() {
          print!(" {:#}", parameter);
        } else {
          print!(" {}", parameter);
        }
      }
      if let Some(doc) = recipe.doc {
        print!(
          " {:padding$}{} {}", "", doc_color.paint("#"), doc_color.paint(doc),
          padding = max_line_width.saturating_sub(
            line_widths.get(name).cloned().unwrap_or(max_line_width)
          )
        );
      }
      println!();
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
        eprintln!("Justfile does not contain recipe `{}`.", name);
        if let Some(suggestion) = justfile.suggest(name) {
          eprintln!("Did you mean `{}`?", suggestion);
        }
        process::exit(EXIT_FAILURE)
      }
    }
  }

  let arguments = if !rest.is_empty() {
    rest
  } else if let Some(recipe) = justfile.first() {
    let min_arguments = recipe.min_arguments();
    if min_arguments > 0 {
      die!("Recipe `{}` cannot be used as default recipe since it requires at least {} argument{}.",
           recipe.name, min_arguments, maybe_s(min_arguments));
    }
    vec![recipe.name]
  } else {
    die!("Justfile contains no recipes.");
  };

  let verbosity = Verbosity::from_flag_occurrences(matches.occurrences_of("VERBOSE"));

  let configuration = Configuration {
    dry_run:   matches.is_present("DRY-RUN"),
    evaluate:  matches.is_present("EVALUATE"),
    highlight: matches.is_present("HIGHLIGHT"),
    quiet:     matches.is_present("QUIET"),
    shell:     matches.value_of("SHELL").unwrap(),
    verbosity,
    color,
    overrides,
  };

  if let Err(error) = InterruptHandler::install() {
    warn!("Failed to set CTRL-C handler: {}", error)
  }

  if let Err(run_error) = justfile.run(
    &invocation_directory,
    &arguments,
    &configuration)
  {
    if !configuration.quiet {
      if color.stderr().active() {
        eprintln!("{:#}", run_error);
      } else {
        eprintln!("{}", run_error);
      }
    }

    process::exit(run_error.code().unwrap_or(EXIT_FAILURE));
  }
}
