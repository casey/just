use crate::common::*;

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches};

pub(crate) const DEFAULT_SHELL: &str = "sh";

pub(crate) struct Config<'a> {
  pub(crate) dry_run: bool,
  pub(crate) evaluate: bool,
  pub(crate) highlight: bool,
  pub(crate) overrides: BTreeMap<&'a str, &'a str>,
  pub(crate) quiet: bool,
  pub(crate) shell: &'a str,
  pub(crate) color: Color,
  pub(crate) verbosity: Verbosity,
  pub(crate) arguments: Vec<&'a str>,
}

impl<'a> Config<'a> {
  pub(crate) fn app() -> App<'static, 'static> {
    let app = App::new(env!("CARGO_PKG_NAME"))
      .help_message("Print help information")
      .version_message("Print version information")
      .setting(AppSettings::ColoredHelp)
      .setting(AppSettings::TrailingVarArg)
      .arg(
        Arg::with_name("ARGUMENTS")
          .multiple(true)
          .help("The recipe(s) to run, defaults to the first recipe in the justfile"),
      )
      .arg(
        Arg::with_name("COLOR")
          .long("color")
          .takes_value(true)
          .possible_values(&["auto", "always", "never"])
          .default_value("auto")
          .help("Print colorful output"),
      )
      .arg(
        Arg::with_name("DRY-RUN")
          .long("dry-run")
          .help("Print what just would do without doing it")
          .conflicts_with("QUIET"),
      )
      .arg(
        Arg::with_name("DUMP")
          .long("dump")
          .help("Print entire justfile"),
      )
      .arg(
        Arg::with_name("EDIT")
          .short("e")
          .long("edit")
          .help("Open justfile with $EDITOR"),
      )
      .arg(
        Arg::with_name("EVALUATE")
          .long("evaluate")
          .help("Print evaluated variables"),
      )
      .arg(
        Arg::with_name("HIGHLIGHT")
          .long("highlight")
          .help("Highlight echoed recipe lines in bold"),
      )
      .arg(
        Arg::with_name("JUSTFILE")
          .short("f")
          .long("justfile")
          .takes_value(true)
          .help("Use <JUSTFILE> as justfile."),
      )
      .arg(
        Arg::with_name("LIST")
          .short("l")
          .long("list")
          .help("List available recipes and their arguments"),
      )
      .arg(
        Arg::with_name("QUIET")
          .short("q")
          .long("quiet")
          .help("Suppress all output")
          .conflicts_with("DRY-RUN"),
      )
      .arg(
        Arg::with_name("SET")
          .long("set")
          .takes_value(true)
          .number_of_values(2)
          .value_names(&["VARIABLE", "VALUE"])
          .multiple(true)
          .help("Set <VARIABLE> to <VALUE>"),
      )
      .arg(
        Arg::with_name("SHELL")
          .long("shell")
          .takes_value(true)
          .default_value(DEFAULT_SHELL)
          .help("Invoke <SHELL> to run recipes"),
      )
      .arg(
        Arg::with_name("SHOW")
          .short("s")
          .long("show")
          .takes_value(true)
          .value_name("RECIPE")
          .help("Show information about <RECIPE>"),
      )
      .arg(
        Arg::with_name("SUMMARY")
          .long("summary")
          .help("List names of available recipes"),
      )
      .arg(
        Arg::with_name("VERBOSE")
          .short("v")
          .long("verbose")
          .multiple(true)
          .help("Use verbose output"),
      )
      .arg(
        Arg::with_name("WORKING-DIRECTORY")
          .short("d")
          .long("working-directory")
          .takes_value(true)
          .help("Use <WORKING-DIRECTORY> as working directory. --justfile must also be set")
          .requires("JUSTFILE"),
      )
      .group(ArgGroup::with_name("EARLY-EXIT").args(&[
        "DUMP",
        "EDIT",
        "LIST",
        "SHOW",
        "SUMMARY",
        "ARGUMENTS",
        "EVALUATE",
      ]));

    if cfg!(feature = "help4help2man") {
      app.version(env!("CARGO_PKG_VERSION")).about(concat!(
        "- Please see ",
        env!("CARGO_PKG_HOMEPAGE"),
        " for more information."
      ))
    } else {
      app
        .version(concat!("v", env!("CARGO_PKG_VERSION")))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(concat!(
          env!("CARGO_PKG_DESCRIPTION"),
          " - ",
          env!("CARGO_PKG_HOMEPAGE")
        ))
    }
  }

  pub(crate) fn from_matches(matches: &'a ArgMatches<'a>) -> Config<'a> {
    let verbosity = Verbosity::from_flag_occurrences(matches.occurrences_of("VERBOSE"));

    let color = match matches.value_of("COLOR").expect("`--color` had no value") {
      "auto" => Color::auto(),
      "always" => Color::always(),
      "never" => Color::never(),
      other => die!(
        "Invalid argument `{}` to --color. This is a bug in just.",
        other
      ),
    };

    let set_count = matches.occurrences_of("SET");
    let mut overrides = BTreeMap::new();
    if set_count > 0 {
      let mut values = matches.values_of("SET").unwrap();
      for _ in 0..set_count {
        overrides.insert(values.next().unwrap(), values.next().unwrap());
      }
    }

    fn is_override(arg: &&str) -> bool {
      arg.chars().skip(1).any(|c| c == '=')
    }

    let raw_arguments: Vec<&str> = matches
      .values_of("ARGUMENTS")
      .map(Iterator::collect)
      .unwrap_or_default();

    for argument in raw_arguments.iter().cloned().take_while(is_override) {
      let i = argument
        .char_indices()
        .skip(1)
        .find(|&(_, c)| c == '=')
        .unwrap()
        .0;

      let name = &argument[..i];
      let value = &argument[i + 1..];

      overrides.insert(name, value);
    }

    let arguments = raw_arguments
      .into_iter()
      .skip_while(is_override)
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

        Some(argument)
      })
      .collect::<Vec<&str>>();

    Config {
      dry_run: matches.is_present("DRY-RUN"),
      evaluate: matches.is_present("EVALUATE"),
      highlight: matches.is_present("HIGHLIGHT"),
      quiet: matches.is_present("QUIET"),
      shell: matches.value_of("SHELL").unwrap(),
      verbosity,
      color,
      overrides,
      arguments,
    }
  }
}

impl<'a> Default for Config<'a> {
  fn default() -> Config<'static> {
    Config {
      dry_run: false,
      evaluate: false,
      highlight: false,
      overrides: empty(),
      arguments: empty(),
      quiet: false,
      shell: DEFAULT_SHELL,
      color: default(),
      verbosity: Verbosity::from_flag_occurrences(0),
    }
  }
}
