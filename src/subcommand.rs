use {
  super::*,
  clap_mangen::Man,
  std::io::{Read, Seek},
  tempfile::tempfile,
};

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
    shell: clap_complete::Shell,
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
  List,
  Man,
  Run {
    arguments: Vec<String>,
    overrides: BTreeMap<String, String>,
  },
  Show {
    name: String,
  },
  Summary,
  Variables,
}

impl Subcommand {
  pub(crate) fn execute<'src>(
    &self,
    config: &Config,
    loader: &'src Loader,
  ) -> Result<(), Error<'src>> {
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
      List => Self::list(config, 0, justfile),
      Show { ref name } => Self::show(config, name, justfile)?,
      Summary => Self::summary(config, justfile),
      Variables => Self::variables(justfile),
      Changelog | Completions { .. } | Edit | Init | Man | Run { .. } => unreachable!(),
    }

    Ok(())
  }

  fn groups(config: &Config, justfile: &Justfile) {
    println!("Recipe groups:");
    for group in justfile.public_groups() {
      println!("{}{group}", config.list_prefix);
    }
  }

  fn run<'src>(
    config: &Config,
    loader: &'src Loader,
    arguments: &[String],
    overrides: &BTreeMap<String, String>,
  ) -> Result<(), Error<'src>> {
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
          Err((err @ Error::UnknownRecipes { .. }, true)) => {
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
  ) -> Result<Compilation<'src>, Error<'src>> {
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
  ) -> Result<(), Error<'src>> {
    let mut recipes = Vec::<&Recipe<Dependency>>::new();
    let mut stack = vec![justfile];
    while let Some(module) = stack.pop() {
      recipes.extend(
        module
          .public_recipes(config.unsorted)
          .iter()
          .filter(|recipe| recipe.min_arguments() == 0),
      );
      stack.extend(module.modules.values());
    }

    if recipes.is_empty() {
      return Err(Error::NoChoosableRecipes);
    }

    let chooser = chooser.map_or_else(|| config::chooser_default(&search.justfile), From::from);

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
      if let Err(io_error) = child
        .stdin
        .as_mut()
        .expect("Child was created with piped stdio")
        .write_all(format!("{}\n", recipe.namepath).as_bytes())
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
      .split_whitespace()
      .map(str::to_owned)
      .collect::<Vec<String>>();

    justfile.run(config, search, overrides, &recipes)
  }

  fn completions(shell: clap_complete::Shell) -> RunResult<'static, ()> {
    use clap_complete::Shell;

    fn replace(haystack: &mut String, needle: &str, replacement: &str) -> RunResult<'static, ()> {
      if let Some(index) = haystack.find(needle) {
        haystack.replace_range(index..index + needle.len(), replacement);
        Ok(())
      } else {
        Err(Error::internal(format!(
          "Failed to find text:\n{needle}\n…in completion script:\n{haystack}"
        )))
      }
    }

    let mut script = {
      let mut tempfile = tempfile().map_err(|io_error| Error::TempfileIo { io_error })?;

      clap_complete::generate(
        shell,
        &mut crate::config::Config::app(),
        env!("CARGO_PKG_NAME"),
        &mut tempfile,
      );

      tempfile
        .rewind()
        .map_err(|io_error| Error::TempfileIo { io_error })?;

      let mut buffer = String::new();

      tempfile
        .read_to_string(&mut buffer)
        .map_err(|io_error| Error::TempfileIo { io_error })?;

      buffer
    };

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
      _ => {}
    }

    println!("{}", script.trim());

    Ok(())
  }

  fn dump(config: &Config, ast: &Ast, justfile: &Justfile) -> Result<(), Error<'static>> {
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

  fn format(config: &Config, search: &Search, src: &str, ast: &Ast) -> Result<(), Error<'static>> {
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

  fn man() -> Result<(), Error<'static>> {
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

  fn list(config: &Config, level: usize, justfile: &Justfile) {
    const MAX_LINE_WIDTH: usize = 30;
    let recipe_aliases = {
      let mut recipe_aliases: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
      if config.no_aliases {
        recipe_aliases
      } else {
        for alias in justfile.aliases.values() {
          if alias.is_private() {
            continue;
          }
          recipe_aliases
            .entry(alias.target.name.lexeme())
            .and_modify(|e| e.push(alias.name.lexeme()))
            .or_insert(vec![alias.name.lexeme()]);
        }
        recipe_aliases
      }
    };

    let line_widths = {
      let mut line_widths: BTreeMap<&str, usize> = BTreeMap::new();

      for recipe in &justfile.public_recipes(config.unsorted) {
        let name = recipe.name.lexeme();

        for name in iter::once(&name).chain(recipe_aliases.get(name).unwrap_or(&Vec::new())) {
          let mut line_width = UnicodeWidthStr::width(*name);

          for parameter in &recipe.parameters {
            line_width += UnicodeWidthStr::width(
              format!(" {}", parameter.color_display(Color::never())).as_str(),
            );
          }

          if line_width <= MAX_LINE_WIDTH {
            line_widths.insert(name, line_width);
          }
        }
      }

      line_widths
    };

    let max_line_width = cmp::min(
      line_widths.values().copied().max().unwrap_or(0),
      MAX_LINE_WIDTH,
    );

    if level == 0 {
      print!("{}", config.list_heading);
    }

    let by_groups = {
      let mut by_groups: BTreeMap<Option<String>, Vec<&Recipe<'_>>> = BTreeMap::new();
      for recipe in justfile.public_recipes(config.unsorted) {
        let groups = recipe.groups();
        if groups.is_empty() {
          by_groups
            .entry(None)
            .and_modify(|e| e.push(recipe))
            .or_insert(vec![recipe]);
        } else {
          for group in groups {
            by_groups
              .entry(Some(group))
              .and_modify(|e| e.push(recipe))
              .or_insert(vec![recipe]);
          }
        }
      }
      by_groups
    };

    let no_recipes_in_group = by_groups.contains_key(&None) && by_groups.len() == 1;

    let print_doc_comment = |doc: &str, padding: usize, doc_color: Color| {
      print!(
        " {:padding$}{} {}",
        "",
        doc_color.paint("#"),
        doc_color.paint(doc),
        padding = padding
      );
    };

    for (group, recipes) in &by_groups {
      if !no_recipes_in_group {
        println!();
        if let Some(group_name) = group {
          println!("[{group_name}]");
        } else {
          println!("(no group)");
        }
      }
      for recipe in recipes {
        let aliases: &[&str] = recipe_aliases
          .get(recipe.name())
          .map_or(&[], |v| v.as_slice());

        let name = recipe.name();
        let doc_color = config.color.stdout().doc();

        for (i, name) in iter::once(&name).chain(aliases).enumerate() {
          print!("{}{name}", config.list_prefix.repeat(level + 1));
          for parameter in &recipe.parameters {
            print!(" {}", parameter.color_display(config.color.stdout()));
          }

          let padding =
            max_line_width.saturating_sub(line_widths.get(name).copied().unwrap_or(max_line_width));
          match (i, recipe.doc) {
            (0, Some(doc)) => print_doc_comment(doc, padding, doc_color),
            (0, None) => (),
            _ => {
              let alias_doc = format!("alias for `{}`", recipe.name);
              print_doc_comment(&alias_doc, padding, doc_color);
            }
          }
          println!();
        }
      }
    }

    for (name, module) in &justfile.modules {
      println!("{}{name}:", config.list_prefix.repeat(level + 1));
      Self::list(config, level + 1, module);
    }
  }

  fn show<'src>(config: &Config, name: &str, justfile: &Justfile<'src>) -> Result<(), Error<'src>> {
    if let Some(alias) = justfile.get_alias(name) {
      let recipe = justfile.get_recipe(alias.target.name.lexeme()).unwrap();
      println!("{alias}");
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

    for recipe in justfile.public_recipes(config.unsorted) {
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
