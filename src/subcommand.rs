use {super::*, clap_mangen::Man};

pub const INIT_JUSTFILE: &str = "\
# https://just.systems

default:
    echo 'Hello, world!'
";

static BACKTICK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("(`.*?`)|(`[^`]*$)").unwrap());

const CHOOSER_CANCELLED_EXIT_STATUS: i32 = 130;

#[derive(PartialEq, Clone, Debug, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Subcommand {
  Changelog,
  Choose {
    chooser: Option<PathBuf>,
  },
  Clean {
    path: Option<Modulepath>,
  },
  Command {
    arguments: Vec<OsString>,
    binary: OsString,
  },
  Completions {
    shell: Shell,
  },
  Dump {
    format: DumpFormat,
  },
  Edit,
  Evaluate {
    format: EvaluateFormat,
    path: Modulepath,
  },
  Format,
  Groups,
  Init,
  List {
    path: Modulepath,
  },
  Man,
  Request {
    request: Request,
  },
  Run {
    arguments: Vec<String>,
  },
  Show {
    path: Modulepath,
  },
  Summary,
  Usage {
    path: Modulepath,
  },
  Variables,
}

impl Default for Subcommand {
  fn default() -> Self {
    Self::Run {
      arguments: Vec::new(),
    }
  }
}

impl Subcommand {
  pub(crate) fn execute<'src>(&self, config: &Config, loader: &'src Loader) -> RunResult<'src> {
    use Subcommand::*;

    match self {
      Changelog => {
        Self::changelog();
        return Ok(());
      }
      Completions { shell } => {
        Self::completions(*shell);
        return Ok(());
      }
      Init => return Self::init(config),
      Man => return Self::man(),
      Request { request } => return Self::request(request),
      _ => {}
    }

    let search = Search::search(config)?;

    if matches!(self, Edit) {
      return Self::edit(&search);
    }

    if matches!(self, Format) {
      return Self::format(config, loader, &search);
    }

    let compilation = Self::compile(config, loader, &search)?;
    let justfile = &compilation.justfile;

    match self {
      Choose { chooser } => {
        Self::choose(
          chooser.as_deref(),
          config,
          justfile,
          &compilation.overrides,
          &search,
        )?;
      }
      Command { .. } | Evaluate { .. } => {
        justfile.run(config, &search, &[], &compilation.overrides)?;
      }
      Clean { path } => Self::clean(config, &search, path.as_ref())?,
      Dump { format } => Self::dump(config, compilation, *format)?,
      Groups => Self::groups(config, justfile),
      List { path } => Self::list(config, justfile, path)?,
      Run { arguments } => Self::run(config, loader, search, compilation, arguments)?,
      Show { path } => Self::show(config, justfile, path)?,
      Summary => Self::summary(config, justfile),
      Usage { path } => Self::usage(config, justfile, path)?,
      Variables => Self::variables(justfile),
      Changelog | Completions { .. } | Edit | Format | Init | Man | Request { .. } => {
        unreachable!()
      }
    }

    Ok(())
  }

  fn groups(config: &Config, justfile: &Justfile) {
    println!("Recipe groups:");
    for group in justfile.public_groups(config) {
      println!("{}{group}", config.list_prefix);
    }
  }

  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }

  fn default_list_module(
    config: &Config,
    justfile: &Justfile,
    arguments: &[String],
  ) -> Option<Modulepath> {
    let path = if arguments.is_empty() {
      Modulepath::default()
    } else if arguments[0].contains(':') {
      if arguments.len() != 1 {
        return None;
      }

      Modulepath::from_argument(&arguments[0]).ok()?
    } else {
      Modulepath::try_from(
        arguments
          .iter()
          .map(String::as_str)
          .collect::<Vec<&str>>()
          .as_slice(),
      )
      .ok()?
    };

    let submodule = justfile.submodule(&path)?;

    (config.default_list || submodule.settings.default_list).then_some(path)
  }

  fn run<'src>(
    config: &Config,
    loader: &'src Loader,
    mut search: Search,
    mut compilation: Compilation<'src>,
    arguments: &[String],
  ) -> RunResult<'src> {
    let starting_parent = search.justfile.parent().as_ref().unwrap().clean();

    loop {
      let justfile = &compilation.justfile;
      let fallback = justfile.settings.fallback
        && matches!(
          config.search_config,
          SearchConfig::FromInvocationDirectory | SearchConfig::FromSearchDirectory { .. }
        );

      if let Some(path) = Self::default_list_module(config, justfile, arguments) {
        return Self::list(config, justfile, &path);
      }

      let result = justfile.run(config, &search, arguments, &compilation.overrides);

      if fallback
        && let Err(err @ (Error::UnknownRecipe { .. } | Error::UnknownSubmodule { .. })) = result
      {
        search = search.search_parent_directory(config).map_err(|_| err)?;

        if config.verbosity.loquacious() {
          eprintln!(
            "Trying {}",
            starting_parent
              .strip_prefix(search.justfile_parent())
              .unwrap()
              .components()
              .map(|_| path::Component::ParentDir)
              .collect::<PathBuf>()
              .join(search.justfile.file_name().unwrap())
              .display()
          );
        }

        compilation = Self::compile(config, loader, &search)?;

        continue;
      }

      if config.allow_missing
        && matches!(
          result,
          Err(Error::UnknownRecipe { .. } | Error::UnknownSubmodule { .. })
        )
      {
        return Ok(());
      }

      return result;
    }
  }

  fn compile<'src>(
    config: &Config,
    loader: &'src Loader,
    search: &Search,
  ) -> RunResult<'src, Compilation<'src>> {
    let compilation = Compiler::compile(config, loader, &search.justfile)?;

    compilation.justfile.check_unstable(config)?;

    if config.verbosity.loud() {
      for warning in &compilation.justfile.warnings {
        eprintln!("{}", warning.color_display(config.color.stderr()));
      }
    }

    Ok(compilation)
  }

  fn changelog() {
    write!(io::stdout(), "{}", include_str!("../CHANGELOG.md")).ok();
  }

  fn choose<'src>(
    chooser: Option<&Path>,
    config: &Config,
    justfile: &Justfile<'src>,
    overrides: &HashMap<Number, String>,
    search: &Search,
  ) -> RunResult<'src> {
    let groups = config.groups.iter().cloned().collect::<BTreeSet<String>>();
    let mut recipes = Vec::<&Recipe>::new();
    let mut stack = vec![justfile];
    while let Some(module) = stack.pop() {
      recipes.extend(module.public_recipes(config).iter().filter(|recipe| {
        recipe.min_arguments() == 0
          && (groups.is_empty() || groups.intersection(&recipe.groups()).next().is_some())
      }));
      stack.extend(module.public_modules(config));
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

    let stdin = child.stdin.as_mut().unwrap();
    for recipe in recipes {
      if let Err(io_error) = writeln!(stdin, "{}", recipe.spaced_recipe_path())
        && io_error.kind() != std::io::ErrorKind::BrokenPipe
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

    if output.status.code() == Some(CHOOSER_CANCELLED_EXIT_STATUS) {
      return Ok(());
    }

    if !output.status.success() {
      return Err(Error::ChooserStatus {
        status: output.status,
        chooser,
      });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
      let arguments = line
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<String>>();

      justfile.run(config, search, &arguments, overrides)?;
    }

    Ok(())
  }

  fn clean(config: &Config, search: &Search, prefix: Option<&Modulepath>) -> RunResult<'static> {
    let entry_re = Regex::new(r"^[0-9a-f]{64}\.json$").unwrap();

    let path = Cache::dir(search);

    let context = |source| Error::FilesystemIo {
      source,
      path: path.clone(),
    };

    let dir = match fs::read_dir(&path) {
      Err(err) if err.kind() == io::ErrorKind::NotFound => {
        if config.verbosity.loud() {
          eprintln!("recipe cache not found");
        }
        return Ok(());
      }
      result => result.map_err(context)?,
    };

    let mut removed = 0;

    for entry in dir {
      let entry = entry.map_err(context)?;

      if !entry_re.is_match(&entry.file_name().to_string_lossy()) {
        continue;
      }

      let path = entry.path();

      if let Some(prefix) = prefix {
        let json = fs::read_to_string(&path).map_err(|source| Error::FilesystemIo {
          source,
          path: path.clone(),
        })?;

        if !json.is_empty() {
          let entry =
            serde_json::from_str::<CacheEntry>(&json).map_err(|source| Error::CacheEntryRead {
              source,
              path: path.clone(),
            })?;

          if !entry.recipe.starts_with(prefix) {
            continue;
          }
        }
      }

      fs::remove_file(&path).map_err(|source| Error::FilesystemIo {
        source,
        path: path.clone(),
      })?;

      removed += 1;
    }

    if let Err(err) = fs::remove_dir(&path)
      && err.kind() != io::ErrorKind::DirectoryNotEmpty
    {
      return Err(context(err));
    }

    if config.verbosity.loud() {
      eprintln!(
        "removed {}",
        Count::numbered_irregular("cache entry", "cache entries", removed)
      );
    }

    Ok(())
  }

  fn completions(shell: Shell) {
    print!("{}", shell.completion_script());
  }

  fn dump(config: &Config, compilation: Compilation, format: DumpFormat) -> RunResult<'static> {
    match format {
      DumpFormat::Json => {
        serde_json::to_writer(io::stdout(), &compilation.justfile)
          .map_err(|source| Error::DumpJson { source })?;
        println!();
      }
      DumpFormat::Just => {
        print!(
          "{}",
          compilation.root_ast().color_display(
            config
              .color
              .with_use_color(UseColor::Never)
              .with_indentation(
                config
                  .indentation
                  .or(compilation.justfile.settings.indentation)
                  .unwrap_or_default()
              )
          )
        );
      }
    }
    Ok(())
  }

  fn edit(search: &Search) -> RunResult<'static> {
    let editor = env::var_os("VISUAL")
      .or_else(|| env::var_os("EDITOR"))
      .unwrap_or_else(|| "vim".into());

    let error = Command::resolve(&editor)
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

  fn format<'src>(config: &Config, loader: &'src Loader, search: &Search) -> RunResult<'src> {
    let root = search.justfile_parent();

    let (path, src) = loader.load(config, root, &search.justfile)?;

    let ast = Parser::parse_source(
      &mut Numerator::new(),
      path,
      &Source::root(&search.justfile),
      src,
    )?;

    let formatted = ast
      .color_display(
        config
          .color
          .with_use_color(UseColor::Never)
          .with_indentation(config.indentation.or(ast.indentation()).unwrap_or_default()),
      )
      .to_string();

    if config.check {
      if formatted == src {
        return Ok(());
      }

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
    } else if search.tempdir.is_some() {
      print!("{formatted}");
      Ok(())
    } else {
      if formatted != src {
        fs::write(&search.justfile, formatted).map_err(|io_error| Error::WriteJustfile {
          justfile: search.justfile.clone(),
          io_error,
        })?;

        if config.verbosity.loud() {
          eprintln!("wrote justfile to `{}`", search.justfile.display());
        }
      }

      Ok(())
    }
  }

  fn init(config: &Config) -> RunResult<'static> {
    let search = Search::init(config)?;

    if filesystem::is_file(&search.justfile)? {
      return Err(Error::InitExists {
        justfile: search.justfile,
      });
    }

    if let Err(io_error) = fs::write(&search.justfile, INIT_JUSTFILE) {
      return Err(Error::WriteJustfile {
        justfile: search.justfile,
        io_error,
      });
    }

    if config.verbosity.loud() {
      eprintln!("wrote justfile to `{}`", search.justfile.display());
    }

    Ok(())
  }

  fn man() -> RunResult<'static> {
    let mut buffer = Vec::<u8>::new();

    Man::new(Arguments::command())
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

  fn request(request: &Request) -> RunResult<'static> {
    let response = match request {
      Request::EnvironmentVariable(key) => Response::EnvironmentVariable(env::var_os(key)),
      #[cfg(not(windows))]
      Request::Signal => {
        let sigset = nix::sys::signal::SigSet::all();

        sigset.thread_block().unwrap();

        let received = sigset.wait().unwrap();

        Response::Signal(received.as_str().into())
      }
    };

    serde_json::to_writer(io::stdout(), &response).map_err(|source| Error::DumpJson { source })?;

    Ok(())
  }

  fn list(config: &Config, mut module: &Justfile, path: &Modulepath) -> RunResult<'static> {
    for name in &path.components {
      if let Some(submodule) = module.modules.get(name) {
        module = submodule;
      } else if module.absent_modules.contains(name) {
        return Err(Error::ModuleAbsent {
          module: module.module_path.join(name),
        });
      } else {
        return Err(Error::UnknownSubmodule {
          path: path.to_string(),
        });
      }
    }

    Self::list_module(config, 0, &config.groups, module)?;

    Ok(())
  }

  fn list_module(
    config: &Config,
    depth: usize,
    groups: &[String],
    module: &Justfile,
  ) -> RunResult<'static> {
    const MAX_WIDTH: usize = 50;

    fn print_doc_and_aliases(
      config: &Config,
      name: &str,
      doc: Option<&str>,
      aliases: &[&str],
      max_signature_width: usize,
      signature_widths: &BTreeMap<&str, usize>,
    ) {
      let color = config.color.stdout();

      let inline_aliases = config.alias_style != AliasStyle::Separate && !aliases.is_empty();

      if inline_aliases || doc.is_some() {
        print!(
          "{:padding$}{}",
          "",
          color.doc().paint("#"),
          padding = max_signature_width.saturating_sub(signature_widths[name]) + 1,
        );
      }

      let print_aliases = || {
        print!(
          " {}",
          color.alias().paint(&format!(
            "[alias{}: {}]",
            if aliases.len() == 1 { "" } else { "es" },
            aliases.join(", ")
          ))
        );
      };

      if inline_aliases && config.alias_style == AliasStyle::Left {
        print_aliases();
      }

      if let Some(doc) = doc {
        print!(" ");
        let mut end = 0;
        for backtick in BACKTICK_RE.find_iter(doc) {
          let prefix = &doc[end..backtick.start()];
          if !prefix.is_empty() {
            print!("{}", color.doc().paint(prefix));
          }
          print!("{}", color.doc_backtick().paint(backtick.as_str()));
          end = backtick.end();
        }

        let suffix = &doc[end..];
        if !suffix.is_empty() {
          print!("{}", color.doc().paint(suffix));
        }
      }

      if inline_aliases && config.alias_style == AliasStyle::Right {
        print_aliases();
      }

      println!();
    }

    let aliases = if config.no_aliases {
      BTreeMap::new()
    } else {
      let mut aliases = BTreeMap::<&str, Vec<&str>>::new();
      for alias in module
        .recipe_aliases
        .values()
        .filter(|alias| alias.is_public())
      {
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
      if !config.list_submodules {
        for submodule in module.public_modules(config) {
          let name = submodule.name();
          signature_widths.insert(name, UnicodeWidthStr::width(format!("{name} ...").as_str()));
        }
      }

      signature_widths
    };

    let max_signature_width = signature_widths
      .values()
      .copied()
      .filter(|width| *width <= MAX_WIDTH)
      .max()
      .unwrap_or(0);

    let list_prefix = config.list_prefix.repeat(depth + 1);

    if !groups.is_empty() {
      let public_groups = module.public_groups(config);
      for group in groups {
        if !public_groups.contains(group) {
          return Err(Error::UnknownGroup {
            group: group.clone(),
          });
        }
      }
    }

    if depth == 0 {
      print!("{}", config.list_heading);
    }

    let recipe_groups = {
      let mut recipe_groups = BTreeMap::<Option<String>, Vec<&Recipe>>::new();
      for recipe in module.public_recipes(config) {
        let recipe_groups_list = recipe.groups();
        if recipe_groups_list.is_empty() {
          recipe_groups.entry(None).or_default().push(recipe);
        } else {
          for group in recipe_groups_list {
            recipe_groups.entry(Some(group)).or_default().push(recipe);
          }
        }
      }
      recipe_groups
    };

    let submodule_groups = {
      let mut submodule_groups = BTreeMap::<Option<String>, Vec<&Justfile>>::new();
      for submodule in module.public_modules(config) {
        let submodule_groups_list = submodule.groups();
        if submodule_groups_list.is_empty() {
          submodule_groups.entry(None).or_default().push(submodule);
        } else {
          for group in submodule_groups_list {
            submodule_groups
              .entry(Some(group.to_string()))
              .or_default()
              .push(submodule);
          }
        }
      }
      submodule_groups
    };

    let mut ordered_groups = if groups.is_empty() {
      module
        .public_groups(config)
        .into_iter()
        .map(Some)
        .collect::<Vec<Option<String>>>()
    } else {
      groups
        .iter()
        .cloned()
        .map(Some)
        .collect::<Vec<Option<String>>>()
    };

    if groups.is_empty()
      && (recipe_groups.contains_key(&None) || submodule_groups.contains_key(&None))
    {
      ordered_groups.insert(0, None);
    }

    let no_groups =
      groups.is_empty() && ordered_groups.len() == 1 && ordered_groups.first() == Some(&None);

    let groups_count = if no_groups { 0 } else { ordered_groups.len() };

    for (i, group) in ordered_groups.into_iter().enumerate() {
      if i > 0 {
        println!();
      }

      if !no_groups && let Some(group) = &group {
        println!(
          "{list_prefix}{}",
          config.color.stdout().group().paint(&format!("[{group}]"))
        );
      }

      if let Some(recipes) = recipe_groups.get(&group) {
        for recipe in recipes {
          let recipe_alias_entries = if config.alias_style == AliasStyle::Separate {
            aliases.get(recipe.name())
          } else {
            None
          };

          for (i, name) in iter::once(&recipe.name())
            .chain(recipe_alias_entries.unwrap_or(&Vec::new()))
            .enumerate()
          {
            let doc = if i == 0 {
              recipe.doc().map(Cow::Borrowed)
            } else {
              Some(Cow::Owned(format!("alias for `{}`", recipe.name)))
            };

            let inline_doc = signature_widths[name] <= MAX_WIDTH
              && doc.as_ref().is_none_or(|doc| doc.lines().count() <= 1);

            if let Some(doc) = &doc
              && !inline_doc
            {
              for line in doc.lines() {
                println!(
                  "{list_prefix}{} {}",
                  config.color.stdout().doc().paint("#"),
                  config.color.stdout().doc().paint(line),
                );
              }
            }

            print!(
              "{list_prefix}{}",
              RecipeSignature { name, recipe }.color_display(config.color.stdout())
            );

            print_doc_and_aliases(
              config,
              name,
              doc.filter(|_| inline_doc).as_deref(),
              aliases
                .get(recipe.name())
                .map(Vec::as_slice)
                .unwrap_or_default(),
              max_signature_width,
              &signature_widths,
            );
          }
        }
      }

      if let Some(submodules) = submodule_groups.get(&group) {
        for (i, submodule) in submodules.iter().enumerate() {
          if config.list_submodules {
            if no_groups && (i + groups_count > 0) {
              println!();
            }
            println!("{list_prefix}{}:", submodule.name());

            Self::list_module(config, depth + 1, &[], submodule)?;
          } else {
            print!("{list_prefix}{} ...", submodule.name());
            print_doc_and_aliases(
              config,
              submodule.name(),
              submodule.doc.as_deref(),
              &[],
              max_signature_width,
              &signature_widths,
            );
          }
        }
      }
    }

    Ok(())
  }

  fn show<'src>(config: &Config, module: &Justfile<'src>, path: &Modulepath) -> RunResult<'src> {
    let (alias, recipe) = Self::resolve_path(module, path, "show")?;

    if let Some(alias) = alias {
      println!("{alias}");
    }

    println!("{}", recipe.color_display(config.color.stdout()));

    Ok(())
  }

  fn summary(config: &Config, justfile: &Justfile) {
    let recipes = justfile.public_recipes_recursive(config);

    for (i, recipe) in recipes.iter().enumerate() {
      if i > 0 {
        print!(" ");
      }
      print!("{}", recipe.recipe_path());
    }
    println!();

    if recipes.is_empty() && config.verbosity.loud() {
      eprintln!("justfile contains no recipes");
    }
  }

  pub(crate) fn takes_arguments(&self) -> bool {
    match self {
      Self::Changelog
      | Self::Dump { .. }
      | Self::Edit
      | Self::Format
      | Self::Init
      | Self::Man
      | Self::Summary
      | Self::Variables => false,
      Self::Choose { .. }
      | Self::Clean { .. }
      | Self::Command { .. }
      | Self::Completions { .. }
      | Self::Evaluate { .. }
      | Self::Groups
      | Self::List { .. }
      | Self::Request { .. }
      | Self::Run { .. }
      | Self::Show { .. }
      | Self::Usage { .. } => true,
    }
  }

  fn usage<'src>(config: &Config, module: &Justfile<'src>, path: &Modulepath) -> RunResult<'src> {
    let (alias, recipe) = Self::resolve_path(module, path, "usage")?;

    if let Some(alias) = alias {
      println!("{alias}");
    }

    println!(
      "{}",
      Usage {
        long: true,
        path,
        recipe,
      }
      .color_display(config.color.stdout()),
    );

    Ok(())
  }

  fn resolve_path<'src, 'run>(
    mut module: &'run Justfile<'src>,
    path: &Modulepath,
    subcommand: &'static str,
  ) -> RunResult<'src, (Option<&'run RecipeAlias<'src>>, &'run Recipe<'src>)> {
    let Some((name, ancestors)) = path.components.split_last() else {
      return Err(Error::RecipeRequired { subcommand });
    };

    for name in ancestors {
      if let Some(submodule) = module.modules.get(name) {
        module = submodule;
      } else if module.absent_modules.contains(name) {
        return Err(Error::ModuleAbsent {
          module: module.module_path.join(name),
        });
      } else {
        return Err(Error::UnknownSubmodule {
          path: path.to_string(),
        });
      }
    }

    if let Some(alias) = module.recipe_alias(name) {
      Ok((Some(alias), &alias.target))
    } else if let Some(recipe) = module.recipe(name) {
      Ok((None, recipe))
    } else {
      Err(Error::UnknownRecipe {
        recipe: name.to_owned(),
        suggestion: module.suggest_recipe(name),
      })
    }
  }

  fn variables(justfile: &Justfile) {
    for (i, (_, assignment)) in justfile
      .assignments
      .iter()
      .filter(|(_, binding)| !binding.private)
      .enumerate()
    {
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
