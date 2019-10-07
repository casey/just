use crate::common::*;

use crate::{interrupt_handler::InterruptHandler, misc::maybe_s};
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

  let app = Config::app();

  let matches = app.get_matches();

  let config = Config::from_matches(&matches);

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
    let current_dir = match env::current_dir() {
      Ok(current_dir) => current_dir,
      Err(io_error) => die!("Error getting current dir: {}", io_error),
    };
    match search::justfile(&current_dir) {
      Ok(name) => {
        if matches.is_present("EDIT") {
          edit(name);
        }
        text = fs::read_to_string(&name)
          .unwrap_or_else(|error| die!("Error reading justfile: {}", error));

        let parent = name.parent().unwrap();

        if let Err(error) = env::set_current_dir(&parent) {
          die!(
            "Error changing directory to {}: {}",
            parent.display(),
            error
          );
        }
      }
      Err(search_error) => die!("{}", search_error),
    }
  }

  let justfile = Parser::parse(&text).unwrap_or_else(|error| {
    if config.color.stderr().active() {
      die!("{:#}", error);
    } else {
      die!("{}", error);
    }
  });

  for warning in &justfile.warnings {
    if config.color.stderr().active() {
      eprintln!("{:#}", warning);
    } else {
      eprintln!("{}", warning);
    }
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

    let doc_color = config.color.stdout().doc();
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
          if config.color.stdout().active() {
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
    if let Some(alias) = justfile.get_alias(name) {
      let recipe = justfile.get_recipe(alias.target).unwrap();
      println!("{}", alias);
      println!("{}", recipe);
      process::exit(EXIT_SUCCESS);
    }
    if let Some(recipe) = justfile.get_recipe(name) {
      println!("{}", recipe);
      process::exit(EXIT_SUCCESS);
    } else {
      eprintln!("Justfile does not contain recipe `{}`.", name);
      if let Some(suggestion) = justfile.suggest(name) {
        eprintln!("Did you mean `{}`?", suggestion);
      }
      process::exit(EXIT_FAILURE)
    }
  }

  let arguments = if !config.arguments.is_empty() {
    config.arguments.clone()
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

  if let Err(error) = InterruptHandler::install() {
    warn!("Failed to set CTRL-C handler: {}", error)
  }

  if let Err(run_error) = justfile.run(&invocation_directory, &arguments, &config) {
    if !config.quiet {
      if config.color.stderr().active() {
        eprintln!("{:#}", run_error);
      } else {
        eprintln!("{}", run_error);
      }
    }

    process::exit(run_error.code().unwrap_or(EXIT_FAILURE));
  }
}
