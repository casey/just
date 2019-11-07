use crate::common::*;

use unicode_width::UnicodeWidthStr;

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum Subcommand<'a> {
  Dump,
  Edit,
  Evaluate,
  Execute,
  List,
  Show { name: &'a str },
  Summary,
}

impl<'a> Subcommand<'a> {
  pub(crate) fn run(self, config: &Config, justfile: Justfile) -> Result<(), i32> {
    use Subcommand::*;

    match self {
      Dump => Self::dump(justfile),
      Edit => {
        eprintln!("Internal error: Subcommand::run unexpectadly invoked on Edit variant!");
        Err(EXIT_FAILURE)
      }
      Execute | Evaluate => Self::execute(config, justfile),
      List => Self::list(config, justfile),
      Show { name } => Self::show(justfile, name),
      Summary => Self::summary(justfile),
    }
  }

  fn dump(justfile: Justfile) -> Result<(), i32> {
    println!("{}", justfile);
    Ok(())
  }

  pub(crate) fn edit(path: &Path) -> Result<(), i32> {
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

  fn list(config: &Config, justfile: Justfile) -> Result<(), i32> {
    // Construct a target to alias map.
    let mut recipe_aliases: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for alias in justfile.aliases.values() {
      if alias.is_private() {
        continue;
      }

      if !recipe_aliases.contains_key(alias.target.lexeme()) {
        recipe_aliases.insert(alias.target.lexeme(), vec![alias.name.lexeme()]);
      } else {
        let aliases = recipe_aliases.get_mut(alias.target.lexeme()).unwrap();
        aliases.push(alias.name.lexeme());
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

    Ok(())
  }

  fn execute(config: &Config, justfile: Justfile) -> Result<(), i32> {
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
      vec![recipe.name()]
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

  fn show(justfile: Justfile, name: &str) -> Result<(), i32> {
    if let Some(alias) = justfile.get_alias(name) {
      let recipe = justfile.get_recipe(alias.target.lexeme()).unwrap();
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

  fn summary(justfile: Justfile) -> Result<(), i32> {
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
    Ok(())
  }
}
