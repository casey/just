use {super::*, clap_mangen::Man};

const INIT_JUSTFILE: &str = "default:\n    echo 'Hello, world!'\n";

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
    shell: completions::Shell,
  },
  Dump,
  Edit,
  Evaluate {
    overrides: BTreeMap<String, String>,
    variable: Option<String>,
  },
  Format,
  Groups,
  Init,
  List {
    path: ModulePath,
  },
  Man,
  Run {
    arguments: Vec<String>,
    overrides: BTreeMap<String, String>,
  },
  Show {
    path: ModulePath,
  },
  Summary,
  Variables,
}

impl Subcommand {
  pub(crate) fn execute<'src>(&self, config: &Config, loader: &'src Loader) -> RunResult<'src> {
    use Subcommand::*;

    match self {
      Changelog => {
        Self::changelog();
        return Ok(());
      }
      Completions { shell } => return Self::completions(*shell),
      Init => return Self::init(config),
      Man => return Self::man(),
      Run {
        arguments,
        overrides,
      } => return Self::run(config, loader, arguments, overrides),
      _ => {}
    }

    let search = Search::find(&config.search_config, &config.invocation_directory)?;

    if let Edit = self {
      return Self::edit(&search);
    }

    let compilation = Self::compile(config, loader, &search)?;
    let justfile = &compilation.justfile;
    let ast = compilation.root_ast();
    let src = compilation.root_src();

    match self {
      Choose { overrides, chooser } => {
        Self::choose(config, justfile, &search, overrides, chooser.as_deref())?;
      }
      Command { overrides, .. } | Evaluate { overrides, .. } => {
        justfile.run(config, &search, overrides, &[])?;
      }
      Dump => Self::dump(config, ast, justfile)?,
      Format => Self::format(config, &search, src, ast)?,
      Groups => Self::groups(config, justfile),
      List { path } => Self::list(config, justfile, path)?,
      Show { path } => Self::show(config, justfile, path)?,
      Summary => Self::summary(config, justfile),
      Variables => Self::variables(justfile),
      Changelog | Completions { .. } | Edit | Init | Man | Run { .. } => unreachable!(),
    }

    Ok(())
  }

  fn groups(config: &Config, justfile: &Justfile) {
    println!("Recipe groups:");
    for group in justfile.public_groups(config) {
      println!("{}{group}", config.list_prefix);
    }
  }

  fn run<'src>(
    config: &Config,
    loader: &'src Loader,
    arguments: &[String],
    overrides: &BTreeMap<String, String>,
  ) -> RunResult<'src> {
    if matches!(
      config.search_config,
      SearchConfig::FromInvocationDirectory | SearchConfig::FromSearchDirectory { .. }
    ) {
      let starting_path = match &config.search_config {
        SearchConfig::FromInvocationDirectory => config.invocation_directory.clone(),
        SearchConfig::FromSearchDirectory { search_directory } => {
          env::current_dir().unwrap().join(search_directory)
        }
        _ => unreachable!(),
      };

      let mut path = starting_path.clone();

      let mut unknown_recipes_errors = None;

      loop {
        let search = match Search::find_next(&path) {
          Err(SearchError::NotFound) => match unknown_recipes_errors {
            Some(err) => return Err(err),
            None => return Err(SearchError::NotFound.into()),
          },
          Err(err) => return Err(err.into()),
          Ok(search) => {
            if config.verbosity.loquacious() && path != starting_path {
              eprintln!(
                "Trying {}",
                starting_path
                  .strip_prefix(path)
                  .unwrap()
                  .components()
                  .map(|_| path::Component::ParentDir)
                  .collect::<PathBuf>()
                  .join(search.justfile.file_name().unwrap())
                  .display()
              );
            }
            search
          }
        };

        match Self::run_inner(config, loader, arguments, overrides, &search) {
          Err((err @ Error::UnknownRecipe { .. }, true)) => {
            match search.justfile.parent().unwrap().parent() {
              Some(parent) => {
                unknown_recipes_errors.get_or_insert(err);
                path = parent.into();
              }
              None => return Err(err),
            }
          }
          result => return result.map_err(|(err, _fallback)| err),
        }
      }
    } else {
      Self::run_inner(
        config,
        loader,
        arguments,
        overrides,
        &Search::find(&config.search_config, &config.invocation_directory)?,
      )
      .map_err(|(err, _fallback)| err)
    }
  }

  fn run_inner<'src>(
    config: &Config,
    loader: &'src Loader,
    arguments: &[String],
    overrides: &BTreeMap<String, String>,
    search: &Search,
  ) -> Result<(), (Error<'src>, bool)> {
    let compilation = Self::compile(config, loader, search).map_err(|err| (err, false))?;
    let justfile = &compilation.justfile;
    justfile
      .run(config, search, overrides, arguments)
      .map_err(|err| (err, justfile.settings.fallback))
  }

  fn compile<'src>(
    config: &Config,
    loader: &'src Loader,
    search: &Search,
  ) -> RunResult<'src, Compilation<'src>> {
    let compilation = Compiler::compile(config.unstable, loader, &search.justfile)?;

    if config.verbosity.loud() {
      for warning in &compilation.justfile.warnings {
        eprintln!("{}", warning.color_display(config.color.stderr()));
      }
    }

    Ok(compilation)
  }

  fn changelog() {
    print!("{}", include_str!("../CHANGELOG.md"));
  }

  fn choose<'src>(
    config: &Config,
    justfile: &Justfile<'src>,
    search: &Search,
    overrides: &BTreeMap<String, String>,
    chooser: Option<&str>,
  ) -> RunResult<'src> {
    let mut recipes = Vec::<&Recipe>::new();
    let mut stack = vec![justfile];
    while let Some(module) = stack.pop() {
      recipes.extend(
        module
          .public_recipes(config)
          .iter()
          .filter(|recipe| recipe.min_arguments() == 0),
      );
      stack.extend(module.modules.values());
    }

    if recipes.is_empty() {
      return Err(Error::NoChoosableRecipes);
    }

    let chooser = if let Some(chooser) = chooser {
      OsString::from(chooser)
    } else {
      let mut chooser = OsString::new();
      chooser.push("fzf --multi --preview 'just --unstable --color always --justfile \"");
      chooser.push(&search.justfile);
      chooser.push("\" --show {}'");
      chooser
    };

    let result = justfile
      .settings
      .shell_command(config)
      .arg(&chooser)
      .current_dir(&search.working_directory)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn();

    let mut child = match result {
      Ok(child) => child,
      Err(io_error) => {
        let (shell_binary, shell_arguments) = justfile.settings.shell(config);
        return Err(Error::ChooserInvoke {
          shell_binary: shell_binary.to_owned(),
          shell_arguments: shell_arguments.join(" "),
          chooser,
          io_error,
        });
      }
    };

    for recipe in recipes {
      writeln!(
        child.stdin.as_mut().unwrap(),
        "{}",
        recipe.namepath.spaced()
      )
      .map_err(|io_error| Error::ChooserWrite {
        io_error,
        chooser: chooser.clone(),
      })?;
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
      .split_whitespace()
      .map(str::to_owned)
      .collect::<Vec<String>>();

    justfile.run(config, search, overrides, &recipes)
  }

  fn completions(shell: completions::Shell) -> RunResult<'static, ()> {
    println!("{}", shell.script()?);
    Ok(())
  }

  fn dump(config: &Config, ast: &Ast, justfile: &Justfile) -> RunResult<'static> {
    match config.dump_format {
      DumpFormat::Json => {
        serde_json::to_writer(io::stdout(), justfile)
          .map_err(|serde_json_error| Error::DumpJson { serde_json_error })?;
        println!();
      }
      DumpFormat::Just => print!("{ast}"),
    }
    Ok(())
  }

  fn edit(search: &Search) -> RunResult<'static> {
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

  fn format(config: &Config, search: &Search, src: &str, ast: &Ast) -> RunResult<'static> {
    config.require_unstable("The `--fmt` command is currently unstable.")?;

    let formatted = ast.to_string();

    if config.check {
      return if formatted == src {
        Ok(())
      } else {
        if !config.verbosity.quiet() {
          use similar::{ChangeTag, TextDiff};

          let diff = TextDiff::configure()
            .algorithm(similar::Algorithm::Patience)
            .diff_lines(src, &formatted);

          for op in diff.ops() {
            for change in diff.iter_changes(op) {
              let (symbol, color) = match change.tag() {
                ChangeTag::Delete => ("-", config.color.stdout().diff_deleted()),
                ChangeTag::Equal => (" ", config.color.stdout()),
                ChangeTag::Insert => ("+", config.color.stdout().diff_added()),
              };

              print!("{}{symbol}{change}{}", color.prefix(), color.suffix());
            }
          }
        }

        Err(Error::FormatCheckFoundDiff)
      };
    }

    fs::write(&search.justfile, formatted).map_err(|io_error| Error::WriteJustfile {
      justfile: search.justfile.clone(),
      io_error,
    })?;

    if config.verbosity.loud() {
      eprintln!("Wrote justfile to `{}`", search.justfile.display());
    }

    Ok(())
  }

  fn init(config: &Config) -> RunResult<'static> {
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

  fn man() -> RunResult<'static> {
    let mut buffer = Vec::<u8>::new();

    Man::new(Config::app())
      .render(&mut buffer)
      .expect("writing to buffer cannot fail");

    let mut stdout = io::stdout().lock();

    stdout
      .write_all(&buffer)
      .map_err(|io_error| Error::StdoutIo { io_error })?;

    stdout
      .flush()
      .map_err(|io_error| Error::StdoutIo { io_error })?;

    Ok(())
  }

  fn list(config: &Config, mut module: &Justfile, path: &ModulePath) -> RunResult<'static> {
    for name in &path.path {
      module = module
        .modules
        .get(name)
        .ok_or_else(|| Error::UnknownSubmodule {
          path: path.to_string(),
        })?;
    }

    Self::list_module(config, module, 0);

    Ok(())
  }

  fn list_module(config: &Config, module: &Justfile, depth: usize) {
    let aliases = if config.no_aliases {
      BTreeMap::new()
    } else {
      let mut aliases = BTreeMap::<&str, Vec<&str>>::new();
      for alias in module.aliases.values().filter(|alias| !alias.is_private()) {
        aliases
          .entry(alias.target.name.lexeme())
          .or_default()
          .push(alias.name.lexeme());
      }
      aliases
    };

    let signature_widths = {
      let mut signature_widths: BTreeMap<&str, usize> = BTreeMap::new();

      for (name, recipe) in &module.recipes {
        if !recipe.is_public() {
          continue;
        }

        for name in iter::once(name).chain(aliases.get(name).unwrap_or(&Vec::new())) {
          signature_widths.insert(
            name,
            UnicodeWidthStr::width(
              RecipeSignature { name, recipe }
                .color_display(Color::never())
                .to_string()
                .as_str(),
            ),
          );
        }
      }

      signature_widths
    };

    let max_signature_width = signature_widths
      .values()
      .copied()
      .filter(|width| *width <= 50)
      .max()
      .unwrap_or(0);

    let list_prefix = config.list_prefix.repeat(depth + 1);

    if depth == 0 {
      print!("{}", config.list_heading);
    }

    let groups = {
      let mut groups = BTreeMap::<Option<String>, Vec<&Recipe>>::new();
      for recipe in module.public_recipes(config) {
        let recipe_groups = recipe.groups();
        if recipe_groups.is_empty() {
          groups.entry(None).or_default().push(recipe);
        } else {
          for group in recipe_groups {
            groups.entry(Some(group)).or_default().push(recipe);
          }
        }
      }
      groups
    };

    let mut ordered = module
      .public_groups(config)
      .into_iter()
      .map(Some)
      .collect::<Vec<Option<String>>>();

    if groups.contains_key(&None) {
      ordered.insert(0, None);
    }

    for (i, group) in ordered.into_iter().enumerate() {
      if i > 0 {
        println!();
      }

      let no_groups = groups.contains_key(&None) && groups.len() == 1;

      if !no_groups {
        print!("{list_prefix}");
        if let Some(group) = &group {
          println!("[{group}]");
        } else {
          println!("(no group)");
        }
      }

      for recipe in groups.get(&group).unwrap() {
        for (i, name) in iter::once(&recipe.name())
          .chain(aliases.get(recipe.name()).unwrap_or(&Vec::new()))
          .enumerate()
        {
          let doc = if i == 0 {
            recipe.doc().map(Cow::Borrowed)
          } else {
            Some(Cow::Owned(format!("alias for `{}`", recipe.name)))
          };

          if let Some(doc) = &doc {
            if doc.lines().count() > 1 {
              for line in doc.lines() {
                println!(
                  "{list_prefix}{} {}",
                  config.color.stdout().doc().paint("#"),
                  config.color.stdout().doc().paint(line),
                );
              }
            }
          }

          print!(
            "{list_prefix}{}",
            RecipeSignature { name, recipe }.color_display(config.color.stdout())
          );

          if let Some(doc) = doc {
            if doc.lines().count() <= 1 {
              print!(
                "{:padding$}{} {}",
                "",
                config.color.stdout().doc().paint("#"),
                config.color.stdout().doc().paint(&doc),
                padding = max_signature_width.saturating_sub(signature_widths[name]) + 1,
              );
            }
          }
          println!();
        }
      }
    }

    if config.list_submodules {
      for (i, submodule) in module.modules(config).into_iter().enumerate() {
        if i + groups.len() > 0 {
          println!();
        }

        println!("{list_prefix}{}:", submodule.name());

        Self::list_module(config, submodule, depth + 1);
      }
    } else {
      for submodule in module.modules(config) {
        println!("{list_prefix}{} ...", submodule.name(),);
      }
    }
  }

  fn show<'src>(
    config: &Config,
    mut module: &Justfile<'src>,
    path: &ModulePath,
  ) -> RunResult<'src> {
    for name in &path.path[0..path.path.len() - 1] {
      module = module
        .modules
        .get(name)
        .ok_or_else(|| Error::UnknownSubmodule {
          path: path.to_string(),
        })?;
    }

    let name = path.path.last().unwrap();

    if let Some(alias) = module.get_alias(name) {
      let recipe = module.get_recipe(alias.target.name.lexeme()).unwrap();
      println!("{alias}");
      println!("{}", recipe.color_display(config.color.stdout()));
      Ok(())
    } else if let Some(recipe) = module.get_recipe(name) {
      println!("{}", recipe.color_display(config.color.stdout()));
      Ok(())
    } else {
      Err(Error::UnknownRecipe {
        recipe: name.to_owned(),
        suggestion: module.suggest_recipe(name),
      })
    }
  }

  fn summary(config: &Config, justfile: &Justfile) {
    let mut printed = 0;
    Self::summary_recursive(config, &mut Vec::new(), &mut printed, justfile);
    println!();

    if printed == 0 && config.verbosity.loud() {
      eprintln!("Justfile contains no recipes.");
    }
  }

  fn summary_recursive<'a>(
    config: &Config,
    components: &mut Vec<&'a str>,
    printed: &mut usize,
    justfile: &'a Justfile,
  ) {
    let path = components.join("::");

    for recipe in justfile.public_recipes(config) {
      if *printed > 0 {
        print!(" ");
      }
      if path.is_empty() {
        print!("{}", recipe.name());
      } else {
        print!("{}::{}", path, recipe.name());
      }
      *printed += 1;
    }

    for (name, module) in &justfile.modules {
      components.push(name);
      Self::summary_recursive(config, components, printed, module);
      components.pop();
    }
  }

  fn variables(justfile: &Justfile) {
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
