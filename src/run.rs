use crate::common::*;

use crate::interrupt_handler::InterruptHandler;
use unicode_width::UnicodeWidthStr;

fn edit<P: AsRef<OsStr>>(path: P) -> Result<(), i32> {
  let editor = match env::var_os("EDITOR") {
    None => {
      eprintln!("Error getting EDITOR environment variable");
      return Err(EXIT_FAILURE);
    }
    Some(editor) => editor,
  };

  let error = Command::new(editor).arg(path).status();

  match error {
    Ok(status) => {
      if status.success() {
        Ok(())
      } else {
        eprintln!("Editor failed: {}", status);
        Err(status.code().unwrap_or(EXIT_FAILURE))
      }
    }
    Err(error) => {
      eprintln!("Failed to invoke editor: {}", error);
      Err(EXIT_FAILURE)
    }
  }
}

pub(crate) const INIT_JUSTFILE: &str = "default:\n\techo 'Hello, world!'\n";

pub fn init() -> Result<(), i32> {
  let current_dir = &env::current_dir().unwrap();
  let root = search::project_root(current_dir)
    .map_err(|_| EXIT_FAILURE)
    .unwrap();

  if let Ok(_) = search::dir(root) {
    eprintln!("Justfile already exists at the project root");
    return Err(EXIT_FAILURE);
  }

  if let Err(e) = fs::write(root.join(search::FILENAME), INIT_JUSTFILE) {
    eprintln!("error writing justfile: {:?}", e);
    return Err(EXIT_FAILURE);
  };

  Ok(())
}

pub fn run() -> Result<(), i32> {
  #[cfg(windows)]
  ansi_term::enable_ansi_support().ok();

  env_logger::Builder::from_env(
    env_logger::Env::new()
      .filter("JUST_LOG")
      .write_style("JUST_LOG_STYLE"),
  )
  .init();

  let app = Config::app();

  let matches = app.get_matches();

  let config = match Config::from_matches(&matches) {
    Ok(config) => config,
    Err(error) => {
      eprintln!("error: {}", error);
      return Err(EXIT_FAILURE);
    }
  };

  let justfile = config.justfile;

  let mut working_directory = config.working_directory.map(PathBuf::from);

  if config.subcommand == Subcommand::Init {
    return init();
  }

  if let (Some(justfile), None) = (justfile, working_directory.as_ref()) {
    let mut justfile = justfile.to_path_buf();

    if !justfile.is_absolute() {
      match justfile.canonicalize() {
        Ok(canonical) => justfile = canonical,
        Err(err) => {
          eprintln!(
            "Could not canonicalize justfile path `{}`: {}",
            justfile.display(),
            err
          );
          return Err(EXIT_FAILURE);
        }
      }
    }

    justfile.pop();

    working_directory = Some(justfile);
  }

  let text;
  if let (Some(justfile), Some(directory)) = (justfile, working_directory) {
    if config.subcommand == Subcommand::Edit {
      return edit(justfile);
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
        if config.subcommand == Subcommand::Edit {
          return edit(name);
        }
        text = match fs::read_to_string(&name) {
          Err(error) => {
            eprintln!("Error reading justfile: {}", error);
            return Err(EXIT_FAILURE);
          }
          Ok(text) => text,
        };

        let parent = name.parent().unwrap();

        if let Err(error) = env::set_current_dir(&parent) {
          eprintln!(
            "Error changing directory to {}: {}",
            parent.display(),
            error
          );
          return Err(EXIT_FAILURE);
        }
      }
      Err(search_error) => {
        eprintln!("{}", search_error);
        return Err(EXIT_FAILURE);
      }
    }
  }

  let justfile = match Parser::parse(&text) {
    Err(error) => {
      if config.color.stderr().active() {
        eprintln!("{:#}", error);
      } else {
        eprintln!("{}", error);
      }
      return Err(EXIT_FAILURE);
    }
    Ok(justfile) => justfile,
  };

  for warning in &justfile.warnings {
    if config.color.stderr().active() {
      eprintln!("{:#}", warning);
    } else {
      eprintln!("{}", warning);
    }
  }

  if config.subcommand == Subcommand::Summary {
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
    return Ok(());
  }

  if config.subcommand == Subcommand::Dump {
    println!("{}", justfile);
    return Ok(());
  }

  if config.subcommand == Subcommand::List {
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

    return Ok(());
  }

  if let Subcommand::Show { name } = config.subcommand {
    if let Some(alias) = justfile.get_alias(name) {
      let recipe = justfile.get_recipe(alias.target).unwrap();
      println!("{}", alias);
      println!("{}", recipe);
      return Ok(());
    }
    if let Some(recipe) = justfile.get_recipe(name) {
      println!("{}", recipe);
      return Ok(());
    } else {
      eprintln!("Justfile does not contain recipe `{}`.", name);
      if let Some(suggestion) = justfile.suggest(name) {
        eprintln!("Did you mean `{}`?", suggestion);
      }
      return Err(EXIT_FAILURE);
    }
  }

  let arguments = if !config.arguments.is_empty() {
    config.arguments.clone()
  } else if let Some(recipe) = justfile.first() {
    let min_arguments = recipe.min_arguments();
    if min_arguments > 0 {
      die!(
        "Recipe `{}` cannot be used as default recipe since it requires at least {} {}.",
        recipe.name,
        min_arguments,
        Count("argument", min_arguments),
      );
    }
    vec![recipe.name]
  } else {
    die!("Justfile contains no recipes.");
  };

  if let Err(error) = InterruptHandler::install() {
    warn!("Failed to set CTRL-C handler: {}", error)
  }

  if let Err(run_error) = justfile.run(&arguments, &config) {
    if !config.quiet {
      if config.color.stderr().active() {
        eprintln!("{:#}", run_error);
      } else {
        eprintln!("{}", run_error);
      }
    }

    return Err(run_error.code().unwrap_or(EXIT_FAILURE));
  }

  Ok(())
}
