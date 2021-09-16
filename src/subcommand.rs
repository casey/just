use crate::common::*;

const INIT_JUSTFILE: &str = "default:\n\techo 'Hello, world!'\n";

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Subcommand {
  Changelog,
  Choose {
    overrides: BTreeMap<String, String>,
    chooser: Option<String>,
  },
  Command {
    arguments: Vec<OsString>,
    binary: OsString,
    overrides: BTreeMap<String, String>,
  },
  Completions {
    shell: String,
  },
  Dump,
  Edit,
  Evaluate {
    overrides: BTreeMap<String, String>,
    variable: Option<String>,
  },
  Format,
  Init,
  List,
  Run {
    overrides: BTreeMap<String, String>,
    arguments: Vec<String>,
  },
  Show {
    name: String,
  },
  Summary,
  Variables,
}

impl Subcommand {
  pub(crate) fn run<'src>(&self, config: &Config, loader: &'src Loader) -> Result<(), Error<'src>> {
    use Subcommand::*;

    match self {
      Changelog => {
        Self::changelog();
        return Ok(());
      }
      Completions { shell } => return Self::completions(&shell),
      Init => return Self::init(config),
      _ => {}
    }

    let search = Search::find(&config.search_config, &config.invocation_directory)?;

    if let Edit = self {
      return Self::edit(&search);
    }

    let src = loader.load(&search.justfile)?;

    let tokens = Lexer::lex(&src)?;
    let ast = Parser::parse(&tokens)?;
    let justfile = Analyzer::analyze(ast.clone())?;

    if config.verbosity.loud() {
      for warning in &justfile.warnings {
        eprintln!("{}", warning.color_display(config.color.stderr()));
      }
    }

    match self {
      Choose { overrides, chooser } => {
        Self::choose(config, justfile, &search, overrides, chooser.as_deref())?
      }
      Command { overrides, .. } => justfile.run(config, &search, overrides, &[])?,
      Dump => Self::dump(ast),
      Evaluate { overrides, .. } => justfile.run(config, &search, overrides, &[])?,
      Format => Self::format(config, ast, &search)?,
      List => Self::list(config, justfile),
      Run {
        arguments,
        overrides,
      } => justfile.run(config, &search, overrides, arguments)?,
      Show { ref name } => Self::show(config, &name, justfile)?,
      Summary => Self::summary(config, justfile),
      Variables => Self::variables(justfile),
      Changelog | Completions { .. } | Edit | Init => unreachable!(),
    }

    Ok(())
  }

  fn changelog() {
    print!("{}", include_str!("../CHANGELOG.md"));
  }

  fn choose<'src>(
    config: &Config,
    justfile: Justfile<'src>,
    search: &Search,
    overrides: &BTreeMap<String, String>,
    chooser: Option<&str>,
  ) -> Result<(), Error<'src>> {
    let recipes = justfile
      .public_recipes(config.unsorted)
      .iter()
      .filter(|recipe| recipe.min_arguments() == 0)
      .cloned()
      .collect::<Vec<&Recipe<Dependency>>>();

    if recipes.is_empty() {
      return Err(Error::NoChoosableRecipes);
    }

    let chooser = chooser
      .map(OsString::from)
      .or_else(|| env::var_os(config::CHOOSER_ENVIRONMENT_KEY))
      .unwrap_or_else(|| OsString::from(config::CHOOSER_DEFAULT));

    let result = justfile
      .settings
      .shell_command(&config)
      .arg(&chooser)
      .current_dir(&search.working_directory)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn();

    let mut child = match result {
      Ok(child) => child,
      Err(io_error) => {
        return Err(Error::ChooserInvoke {
          shell_binary: justfile.settings.shell_binary(&config).to_owned(),
          shell_arguments: justfile.settings.shell_arguments(&config).join(" "),
          chooser,
          io_error,
        });
      }
    };

    for recipe in recipes {
      if let Err(io_error) = child
        .stdin
        .as_mut()
        .expect("Child was created with piped stdio")
        .write_all(format!("{}\n", recipe.name).as_bytes())
      {
        return Err(Error::ChooserWrite { io_error, chooser });
      }
    }

    let output = match child.wait_with_output() {
      Ok(output) => output,
      Err(io_error) => {
        return Err(Error::ChooserRead { io_error, chooser });
      }
    };

    if !output.status.success() {
      return Err(Error::ChooserStatus {
        status: output.status,
        chooser,
      });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let recipes = stdout
      .trim()
      .split_whitespace()
      .map(str::to_owned)
      .collect::<Vec<String>>();

    justfile.run(config, search, overrides, &recipes)
  }

  fn completions(shell: &str) -> RunResult<'static, ()> {
    use clap::Shell;

    fn replace(haystack: &mut String, needle: &str, replacement: &str) -> RunResult<'static, ()> {
      if let Some(index) = haystack.find(needle) {
        haystack.replace_range(index..index + needle.len(), replacement);
        Ok(())
      } else {
        Err(Error::internal(format!(
          "Failed to find text:\n{}\n…in completion script:\n{}",
          needle, haystack
        )))
      }
    }

    let shell = shell
      .parse::<Shell>()
      .expect("Invalid value for clap::Shell");

    let buffer = Vec::new();
    let mut cursor = Cursor::new(buffer);
    Config::app().gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut cursor);
    let buffer = cursor.into_inner();
    let mut script = String::from_utf8(buffer).expect("Clap completion not UTF-8");

    match shell {
      Shell::Bash => {
        for (needle, replacement) in completions::BASH_COMPLETION_REPLACEMENTS {
          replace(&mut script, needle, replacement)?;
        }
      }
      Shell::Fish => {
        script.insert_str(0, completions::FISH_RECIPE_COMPLETIONS);
      }
      Shell::PowerShell => {
        for (needle, replacement) in completions::POWERSHELL_COMPLETION_REPLACEMENTS {
          replace(&mut script, needle, replacement)?;
        }
      }

      Shell::Zsh => {
        for (needle, replacement) in completions::ZSH_COMPLETION_REPLACEMENTS {
          replace(&mut script, needle, replacement)?;
        }
      }
      Shell::Elvish => {}
    }

    println!("{}", script.trim());

    Ok(())
  }

  fn dump(ast: Ast) {
    print!("{}", ast);
  }

  fn edit(search: &Search) -> Result<(), Error<'static>> {
    let editor = env::var_os("VISUAL")
      .or_else(|| env::var_os("EDITOR"))
      .unwrap_or_else(|| "vim".into());

    let error = Command::new(&editor)
      .current_dir(&search.working_directory)
      .arg(&search.justfile)
      .status();

    let status = match error {
      Err(io_error) => return Err(Error::EditorInvoke { editor, io_error }),
      Ok(status) => status,
    };

    if !status.success() {
      return Err(Error::EditorStatus { editor, status });
    }

    Ok(())
  }

  fn format(config: &Config, ast: Ast, search: &Search) -> Result<(), Error<'static>> {
    config.require_unstable("The `--fmt` command is currently unstable.")?;

    if let Err(io_error) =
      File::create(&search.justfile).and_then(|mut file| write!(file, "{}", ast))
    {
      Err(Error::WriteJustfile {
        justfile: search.justfile.clone(),
        io_error,
      })
    } else {
      if config.verbosity.loud() {
        eprintln!("Wrote justfile to `{}`", search.justfile.display());
      }
      Ok(())
    }
  }

  fn init(config: &Config) -> Result<(), Error<'static>> {
    let search = Search::init(&config.search_config, &config.invocation_directory)?;

    if search.justfile.is_file() {
      Err(Error::InitExists {
        justfile: search.justfile,
      })
    } else if let Err(io_error) = fs::write(&search.justfile, INIT_JUSTFILE) {
      Err(Error::WriteJustfile {
        justfile: search.justfile,
        io_error,
      })
    } else {
      if config.verbosity.loud() {
        eprintln!("Wrote justfile to `{}`", search.justfile.display());
      }
      Ok(())
    }
  }

  fn list(config: &Config, justfile: Justfile) {
    // Construct a target to alias map.
    let mut recipe_aliases: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for alias in justfile.aliases.values() {
      if alias.is_private() {
        continue;
      }

      if !recipe_aliases.contains_key(alias.target.name.lexeme()) {
        recipe_aliases.insert(alias.target.name.lexeme(), vec![alias.name.lexeme()]);
      } else {
        let aliases = recipe_aliases.get_mut(alias.target.name.lexeme()).unwrap();
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
          line_width += UnicodeWidthStr::width(
            format!(" {}", parameter.color_display(Color::never())).as_str(),
          );
        }

        if line_width <= 30 {
          line_widths.insert(name, line_width);
        }
      }
    }

    let max_line_width = cmp::min(line_widths.values().cloned().max().unwrap_or(0), 30);

    let doc_color = config.color.stdout().doc();
    print!("{}", config.list_heading);

    for recipe in justfile.public_recipes(config.unsorted) {
      let name = recipe.name();

      for (i, name) in iter::once(&name)
        .chain(recipe_aliases.get(name).unwrap_or(&Vec::new()))
        .enumerate()
      {
        print!("{}{}", config.list_prefix, name);
        for parameter in &recipe.parameters {
          print!(" {}", parameter.color_display(config.color.stdout()));
        }

        // Declaring this outside of the nested loops will probably be more efficient,
        // but it creates all sorts of lifetime issues with variables inside the loops.
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
          _ => {
            let alias_doc = format!("alias for `{}`", recipe.name);
            print_doc(&alias_doc);
          }
        }
        println!();
      }
    }
  }

  fn show<'src>(config: &Config, name: &str, justfile: Justfile<'src>) -> Result<(), Error<'src>> {
    if let Some(alias) = justfile.get_alias(name) {
      let recipe = justfile.get_recipe(alias.target.name.lexeme()).unwrap();
      println!("{}", alias);
      println!("{}", recipe.color_display(config.color.stdout()));
      Ok(())
    } else if let Some(recipe) = justfile.get_recipe(name) {
      println!("{}", recipe.color_display(config.color.stdout()));
      Ok(())
    } else {
      Err(Error::UnknownRecipes {
        recipes: vec![name.to_owned()],
        suggestion: justfile.suggest_recipe(name),
      })
    }
  }

  fn summary(config: &Config, justfile: Justfile) {
    if justfile.count() == 0 {
      if config.verbosity.loud() {
        eprintln!("Justfile contains no recipes.");
      }
    } else {
      let summary = justfile
        .public_recipes(config.unsorted)
        .iter()
        .map(|recipe| recipe.name())
        .collect::<Vec<&str>>()
        .join(" ");
      println!("{}", summary);
    }
  }

  fn variables(justfile: Justfile) {
    for (i, (_, assignment)) in justfile.assignments.iter().enumerate() {
      if i > 0 {
        print!(" ");
      }
      print!("{}", assignment.name);
    }
    println!();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn init_justfile() {
    testing::compile(INIT_JUSTFILE);
  }
}
