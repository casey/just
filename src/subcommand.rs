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
      Format => Self::format(config, &search, src, ast, justfile)?,
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
        SearchConfig::FromSearchDirectory { search_directory } => config
          .invocation_directory
          .join(search_directory)
          .lexiclean(),
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
    let compilation = Compiler::compile(loader, &search.justfile)?;

    compilation.justfile.check_unstable(config)?;

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

  fn format(
    config: &Config,
    search: &Search,
    src: &str,
    ast: &Ast,
    justfile: &Justfile,
  ) -> RunResult<'static> {
    config.require_unstable(justfile, UnstableFeature::FormatSubcommand)?;

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
    let mut result = Ok(());
    for name in &path.path {
      let submodule = module
        .modules
        .get(name)
        .ok_or_else(|| Error::UnknownSubmodule {
          path: path.to_string(),
        });
      match submodule {
        Ok(submodule) => module = submodule,
        Err(err) => {
          result = Err(err);
          break;
        }
      }
    }

    // default is to check submodules, otherwise we check groups
    if result.is_err() {
      return Self::list_group_recursive(config, module, path, 0, config.list_prefix.as_str());
    }

    Self::list_module(config, module, 0);

    Ok(())
  }

  fn format_entries<'src>(
    config: &Config,
    entries: &Vec<ListEntry<'src, '_>>,
    signature_widths: &SignatureWidths<'src>,
    list_prefix: &str,
    include_prefix: bool,
  ) {
    for entry in entries {
      for (i, name) in iter::once(entry.recipe.name())
        .chain(entry.aliases.iter().copied())
        .enumerate()
      {
        let doc = if i == 0 {
          entry.recipe.doc().map(Cow::Borrowed)
        } else {
          Some(Cow::Owned(format!("alias for `{}`", entry.recipe.name)))
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
          "{list_prefix}{}{}",
          {
            if include_prefix {
              &entry.prefix
            } else {
              ""
            }
          },
          RecipeSignature {
            name,
            recipe: entry.recipe
          }
          .color_display(config.color.stdout())
        );

        Self::format_doc(config, name, doc.as_deref(), signature_widths);
      }
    }
  }

  fn list_group_recursive(
    config: &Config,
    module: &Justfile,
    path: &ModulePath,
    depth: usize,
    prefix: &str,
  ) -> RunResult<'static> {
    let mut entries = Vec::new();
    let mut signature_widths = SignatureWidths::empty();
    let mut queue = Vec::new();
    queue.push((String::new(), module));
    while let Some((prefix, module)) = queue.pop() {
      if config.list_submodules {
        let name = module.name();
        queue.append(
          &mut iter::repeat(if name.is_empty() {
            String::new()
          } else {
            format!("{prefix}{name}::")
          })
          .zip(module.modules(config).into_iter())
          .collect(),
        );
      }
      let target_name = path.path.first().unwrap().to_string();
      let group = module.find_public_group(&target_name, config);
      let aliases = module.aliases(config);
      for recipe in &group {
        let name = module.name();
        let entry = ListEntry::from_recipe(
          recipe,
          if name.is_empty() {
            String::new()
          } else {
            format!("{prefix}{}::", module.name())
          },
          aliases.get(recipe.name()).unwrap_or(&Vec::new()).clone(),
        );
        signature_widths.add_entry(&entry);
        entries.push(entry);
      }
    }

    if entries.is_empty() {
      return Err(Error::UnknownSubmoduleGroup {
        path: path.to_string(),
      });
    }

    let list_prefix = prefix;

    if depth == 0 {
      print!("{}", config.list_heading);
    }

    Self::format_entries(config, &entries, &signature_widths, list_prefix, true);

    Ok(())
  }

  fn format_doc(
    config: &Config,
    name: &str,
    doc: Option<&str>,
    signature_widths: &SignatureWidths,
  ) {
    if let Some(doc) = doc {
      if !doc.is_empty() && doc.lines().count() <= 1 {
        print!(
          "{:padding$}{} {}",
          "",
          config.color.stdout().doc().paint("#"),
          config.color.stdout().doc().paint(doc),
          padding = signature_widths
            .max_width
            .saturating_sub(signature_widths.widths[name])
            + 1,
        );
      }
    }
    println!();
  }

  fn list_module(config: &Config, module: &Justfile, depth: usize) {
    let mut signature_widths = SignatureWidths::empty();
    let recipe_groups = module.public_group_map(config);

    recipe_groups
      .values()
      .for_each(|entries| signature_widths.add_entries(entries));
    if !config.list_submodules {
      for (name, _) in &module.modules {
        signature_widths
          .add_string_custom_width(name, UnicodeWidthStr::width(format!("{name} ...").as_str()));
      }
    }

    let list_prefix = config.list_prefix.repeat(depth + 1);

    if depth == 0 {
      print!("{}", config.list_heading);
    }

    let submodule_groups = {
      let mut groups = BTreeMap::<Option<String>, Vec<&Justfile>>::new();
      for submodule in module.modules(config) {
        let submodule_groups = submodule.groups();
        if submodule_groups.is_empty() {
          groups.entry(None).or_default().push(submodule);
        } else {
          for group in submodule_groups {
            groups
              .entry(Some(group.to_string()))
              .or_default()
              .push(submodule);
          }
        }
      }
      groups
    };

    let mut ordered_groups = module
      .public_groups(config)
      .into_iter()
      .map(Some)
      .collect::<Vec<Option<String>>>();

    if recipe_groups.contains_key(&None) || submodule_groups.contains_key(&None) {
      ordered_groups.insert(0, None);
    }

    let no_groups = ordered_groups.len() == 1 && ordered_groups.first() == Some(&None);
    let mut groups_count = 0;
    if !no_groups {
      groups_count = ordered_groups.len();
    }

    for (i, group) in ordered_groups.into_iter().enumerate() {
      if i > 0 {
        println!();
      }

      if !no_groups {
        if let Some(group) = &group {
          println!("{list_prefix}[{group}]");
        }
      }

      if let Some(entries) = recipe_groups.get(&group) {
        Self::format_entries(config, entries, &signature_widths, &list_prefix, false);
      }

      if let Some(submodules) = submodule_groups.get(&group) {
        for (i, submodule) in submodules.iter().enumerate() {
          if config.list_submodules {
            if no_groups && (i + groups_count > 0) {
              println!();
            }
            println!("{list_prefix}{}:", submodule.name());

            Self::list_module(config, submodule, depth + 1);
          } else {
            print!("{list_prefix}{} ...", submodule.name());
            Self::format_doc(
              config,
              submodule.name(),
              submodule.doc.as_deref(),
              &signature_widths,
            );
          }
        }
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
