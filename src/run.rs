use crate::common::*;

use crate::configuration::DEFAULT_SHELL;
use crate::interrupt_handler::InterruptHandler;
use crate::misc::maybe_s;
use clap::{App, AppSettings, Arg, ArgGroup};
use std::{convert, ffi};
use unicode_width::UnicodeWidthStr;

#[cfg(windows)]
use ansi_term::enable_ansi_support;

fn edit<P: convert::AsRef<ffi::OsStr>>(path: P) -> ! {
  let editor =
    env::var_os("EDITOR").unwrap_or_else(|| die!("Error getting EDITOR environment variable"));

  let error = Command::new(editor).arg(path).status();

  match error {
    Ok(status) => process::exit(status.code().unwrap_or(EXIT_FAILURE)),
    Err(error) => die!("Failed to invoke editor: {}", error),
  }
}

pub fn run() {
  #[cfg(windows)]
  enable_ansi_support().ok();

  env_logger::Builder::from_env(
    env_logger::Env::new()
      .filter("JUST_LOG")
      .write_style("JUST_LOG_STYLE"),
  )
  .init();

  let invocation_directory =
    env::current_dir().map_err(|e| format!("Error getting current directory: {}", e));

  let matches = App::new(env!("CARGO_PKG_NAME"))
    .version(concat!("v", env!("CARGO_PKG_VERSION")))
    .author(env!("CARGO_PKG_AUTHORS"))
    .about(concat!(
      env!("CARGO_PKG_DESCRIPTION"),
      " - ",
      env!("CARGO_PKG_HOMEPAGE")
    ))
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
    ]))
    .get_matches();

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

  let rest = raw_arguments
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

  let justfile = matches.value_of("JUSTFILE").map(Path::new);
  let mut working_directory = matches.value_of("WORKING-DIRECTORY").map(PathBuf::from);

  if let (Some(justfile), None) = (justfile, working_directory.as_ref()) {
    let mut justfile = justfile.to_path_buf();

    if !justfile.is_absolute() {
      match justfile.canonicalize() {
        Ok(canonical) => justfile = canonical,
        Err(err) => die!(
          "Could not canonicalize justfile path `{}`: {}",
          justfile.display(),
          err
        ),
      }
    }

    justfile.pop();

    working_directory = Some(justfile);
  }

  let text;
  if let (Some(justfile), Some(directory)) = (justfile, working_directory) {
    if matches.is_present("EDIT") {
      edit(justfile);
    }

    text = fs::read_to_string(justfile)
      .unwrap_or_else(|error| die!("Error reading justfile: {}", error));

    if let Err(error) = env::set_current_dir(&directory) {
      die!(
        "Error changing directory to {}: {}",
        directory.display(),
        error
      );
    }
  } else {
    use crate::search::search;
    match search(
      env::current_dir()
        .as_ref()
        .expect("Error getting current directory"),
    ) {
      Ok(name) => {
        if matches.is_present("EDIT") {
          edit(name);
        }
        text = fs::read_to_string(name)
          .unwrap_or_else(|error| die!("Error reading justfile: {}", error));
      }
      Err(search_error) => die!("{}", search_error),
    }
  }

  let justfile = Parser::parse(&text).unwrap_or_else(|error| {
    if color.stderr().active() {
      die!("{:#}", error);
    } else {
      die!("{}", error);
    }
  });

  if justfile.deprecated_equals {
    let warning = color.warning().stderr();
    let message = color.message().stderr();

    eprintln!(
      "{}",
      warning.paint(
        "warning: `=` in assignments, exports, and aliases is being phased out on favor of `:=`"
      )
    );

    eprintln!(
      "{}",
      message
        .paint("Please see this issue for more details: https://github.com/casey/just/issues/379")
    );
  }

  if matches.is_present("SUMMARY") {
    if justfile.count() == 0 {
      eprintln!("Justfile contains no recipes.");
    } else {
      let summary = justfile
        .recipes
        .iter()
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
    // Construct a target to alias map.
    let mut recipe_aliases: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for alias in justfile.aliases.values() {
      if alias.private {
        continue;
      }

      if !recipe_aliases.contains_key(alias.target) {
        recipe_aliases.insert(alias.target, vec![alias.name]);
      } else {
        let aliases = recipe_aliases.get_mut(alias.target).unwrap();
        aliases.push(alias.name);
      }
    }

    let mut line_widths: BTreeMap<&str, usize> = BTreeMap::new();

    for (name, recipe) in &justfile.recipes {
      if recipe.private {
        continue;
      }

      for name in iter::once(name).chain(recipe_aliases.get(name).unwrap_or(&Vec::new())) {
        let mut line_width = UnicodeWidthStr::width(*name);

        for parameter in &recipe.parameters {
          line_width += UnicodeWidthStr::width(format!(" {}", parameter).as_str());
        }

        if line_width <= 30 {
          line_widths.insert(name, line_width);
        }
      }
    }

    let max_line_width = cmp::min(line_widths.values().cloned().max().unwrap_or(0), 30);

    let doc_color = color.stdout().doc();
    println!("Available recipes:");

    for (name, recipe) in &justfile.recipes {
      if recipe.private {
        continue;
      }

      let alias_doc = format!("alias for `{}`", recipe.name);

      for (i, name) in iter::once(name)
        .chain(recipe_aliases.get(name).unwrap_or(&Vec::new()))
        .enumerate()
      {
        print!("    {}", name);
        for parameter in &recipe.parameters {
          if color.stdout().active() {
            print!(" {:#}", parameter);
          } else {
            print!(" {}", parameter);
          }
        }

        // Declaring this outside of the nested loops will probably be more efficient, but
        // it creates all sorts of lifetime issues with variables inside the loops.
        // If this is inlined like the docs say, it shouldn't make any difference.
        let print_doc = |doc| {
          print!(
            " {:padding$}{} {}",
            "",
            doc_color.paint("#"),
            doc_color.paint(doc),
            padding = max_line_width
              .saturating_sub(line_widths.get(name).cloned().unwrap_or(max_line_width))
          );
        };

        match (i, recipe.doc) {
          (0, Some(doc)) => print_doc(doc),
          (0, None) => (),
          _ => print_doc(&alias_doc),
        }
        println!();
      }
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
      die!(
        "Recipe `{}` cannot be used as default recipe since it requires at least {} argument{}.",
        recipe.name,
        min_arguments,
        maybe_s(min_arguments)
      );
    }
    vec![recipe.name]
  } else {
    die!("Justfile contains no recipes.");
  };

  let verbosity = Verbosity::from_flag_occurrences(matches.occurrences_of("VERBOSE"));

  let configuration = Configuration {
    dry_run: matches.is_present("DRY-RUN"),
    evaluate: matches.is_present("EVALUATE"),
    highlight: matches.is_present("HIGHLIGHT"),
    quiet: matches.is_present("QUIET"),
    shell: matches.value_of("SHELL").unwrap(),
    verbosity,
    color,
    overrides,
  };

  if let Err(error) = InterruptHandler::install() {
    warn!("Failed to set CTRL-C handler: {}", error)
  }

  if let Err(run_error) = justfile.run(&invocation_directory, &arguments, &configuration) {
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
