use crate::common::*;

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, ArgSettings};

// These three strings should be kept in sync:
pub(crate) const CHOOSER_DEFAULT: &str = "fzf";
pub(crate) const CHOOSER_ENVIRONMENT_KEY: &str = "JUST_CHOOSER";
pub(crate) const CHOOSE_HELP: &str = "Select one or more recipes to run using a binary. If \
                                      `--chooser` is not passed the chooser defaults to the value \
                                      of $JUST_CHOOSER, falling back to `fzf`";

pub(crate) const DEFAULT_SHELL: &str = "sh";
pub(crate) const DEFAULT_SHELL_ARG: &str = "-cu";
pub(crate) const INIT_JUSTFILE: &str = "default:\n\techo 'Hello, world!'\n";

#[derive(Debug, PartialEq)]
pub(crate) struct Config {
  pub(crate) color:                Color,
  pub(crate) dry_run:              bool,
  pub(crate) highlight:            bool,
  pub(crate) invocation_directory: PathBuf,
  pub(crate) list_heading:         String,
  pub(crate) list_prefix:          String,
  pub(crate) load_dotenv:          bool,
  pub(crate) search_config:        SearchConfig,
  pub(crate) shell:                String,
  pub(crate) shell_args:           Vec<String>,
  pub(crate) shell_present:        bool,
  pub(crate) shell_command:        bool,
  pub(crate) subcommand:           Subcommand,
  pub(crate) unsorted:             bool,
  pub(crate) verbosity:            Verbosity,
}

mod cmd {
  pub(crate) const CHOOSE: &str = "CHOOSE";
  pub(crate) const COMPLETIONS: &str = "COMPLETIONS";
  pub(crate) const DUMP: &str = "DUMP";
  pub(crate) const EDIT: &str = "EDIT";
  pub(crate) const EVALUATE: &str = "EVALUATE";
  pub(crate) const INIT: &str = "INIT";
  pub(crate) const LIST: &str = "LIST";
  pub(crate) const SHOW: &str = "SHOW";
  pub(crate) const SUMMARY: &str = "SUMMARY";
  pub(crate) const VARIABLES: &str = "VARIABLES";
  pub(crate) const COMMAND: &str = "COMMAND";

  pub(crate) const ALL: &[&str] = &[
    CHOOSE,
    COMMAND,
    COMPLETIONS,
    DUMP,
    EDIT,
    EVALUATE,
    INIT,
    LIST,
    SHOW,
    SUMMARY,
    VARIABLES,
  ];

  pub(crate) const ARGLESS: &[&str] = &[
    COMPLETIONS,
    DUMP,
    EDIT,
    INIT,
    LIST,
    SHOW,
    SUMMARY,
    VARIABLES,
  ];
}

mod arg {
  pub(crate) const ARGUMENTS: &str = "ARGUMENTS";
  pub(crate) const CHOOSER: &str = "CHOOSER";
  pub(crate) const CLEAR_SHELL_ARGS: &str = "CLEAR-SHELL-ARGS";
  pub(crate) const COLOR: &str = "COLOR";
  pub(crate) const DRY_RUN: &str = "DRY-RUN";
  pub(crate) const HIGHLIGHT: &str = "HIGHLIGHT";
  pub(crate) const JUSTFILE: &str = "JUSTFILE";
  pub(crate) const LIST_HEADING: &str = "LIST-HEADING";
  pub(crate) const LIST_PREFIX: &str = "LIST-PREFIX";
  pub(crate) const NO_DOTENV: &str = "NO-DOTENV";
  pub(crate) const NO_HIGHLIGHT: &str = "NO-HIGHLIGHT";
  pub(crate) const QUIET: &str = "QUIET";
  pub(crate) const SET: &str = "SET";
  pub(crate) const SHELL: &str = "SHELL";
  pub(crate) const SHELL_ARG: &str = "SHELL-ARG";
  pub(crate) const SHELL_COMMAND: &str = "SHELL-COMMAND";
  pub(crate) const UNSORTED: &str = "UNSORTED";
  pub(crate) const VERBOSE: &str = "VERBOSE";
  pub(crate) const WORKING_DIRECTORY: &str = "WORKING-DIRECTORY";

  pub(crate) const COLOR_ALWAYS: &str = "always";
  pub(crate) const COLOR_AUTO: &str = "auto";
  pub(crate) const COLOR_NEVER: &str = "never";
  pub(crate) const COLOR_VALUES: &[&str] = &[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER];
}

impl Config {
  pub(crate) fn app() -> App<'static, 'static> {
    let app = App::new(env!("CARGO_PKG_NAME"))
      .help_message("Print help information")
      .version_message("Print version information")
      .setting(AppSettings::ColoredHelp)
      .setting(AppSettings::TrailingVarArg)
      .arg(
        Arg::with_name(arg::CHOOSER)
          .long("chooser")
          .takes_value(true)
          .help("Override binary invoked by `--choose`"),
      )
      .arg(
        Arg::with_name(arg::COLOR)
          .long("color")
          .takes_value(true)
          .possible_values(arg::COLOR_VALUES)
          .default_value(arg::COLOR_AUTO)
          .help("Print colorful output"),
      )
      .arg(
        Arg::with_name(arg::DRY_RUN)
          .long("dry-run")
          .help("Print what just would do without doing it")
          .conflicts_with(arg::QUIET),
      )
      .arg(
        Arg::with_name(arg::HIGHLIGHT)
          .long("highlight")
          .help("Highlight echoed recipe lines in bold")
          .overrides_with(arg::NO_HIGHLIGHT),
      )
      .arg(
        Arg::with_name(arg::LIST_HEADING)
          .long("list-heading")
          .help("Print <TEXT> before list")
          .value_name("TEXT")
          .takes_value(true),
      )
      .arg(
        Arg::with_name(arg::LIST_PREFIX)
          .long("list-prefix")
          .help("Print <TEXT> before each list item")
          .value_name("TEXT")
          .takes_value(true),
      )
      .arg(
        Arg::with_name(arg::NO_DOTENV)
          .long("no-dotenv")
          .help("Don't load `.env` file"),
      )
      .arg(
        Arg::with_name(arg::NO_HIGHLIGHT)
          .long("no-highlight")
          .help("Don't highlight echoed recipe lines in bold")
          .overrides_with(arg::HIGHLIGHT),
      )
      .arg(
        Arg::with_name(arg::JUSTFILE)
          .short("f")
          .long("justfile")
          .takes_value(true)
          .help("Use <JUSTFILE> as justfile"),
      )
      .arg(
        Arg::with_name(arg::QUIET)
          .short("q")
          .long("quiet")
          .help("Suppress all output")
          .conflicts_with(arg::DRY_RUN),
      )
      .arg(
        Arg::with_name(arg::SET)
          .long("set")
          .takes_value(true)
          .number_of_values(2)
          .value_names(&["VARIABLE", "VALUE"])
          .multiple(true)
          .help("Override <VARIABLE> with <VALUE>"),
      )
      .arg(
        Arg::with_name(arg::SHELL)
          .long("shell")
          .takes_value(true)
          .default_value(DEFAULT_SHELL)
          .help("Invoke <SHELL> to run recipes"),
      )
      .arg(
        Arg::with_name(arg::SHELL_ARG)
          .long("shell-arg")
          .takes_value(true)
          .multiple(true)
          .number_of_values(1)
          .default_value(DEFAULT_SHELL_ARG)
          .allow_hyphen_values(true)
          .overrides_with(arg::CLEAR_SHELL_ARGS)
          .help("Invoke shell with <SHELL-ARG> as an argument"),
      )
      .arg(
        Arg::with_name(arg::SHELL_COMMAND)
          .long("shell-command")
          .requires(cmd::COMMAND)
          .help("Invoke <COMMAND> with the shell used to run recipe lines and backticks"),
      )
      .arg(
        Arg::with_name(arg::CLEAR_SHELL_ARGS)
          .long("clear-shell-args")
          .overrides_with(arg::SHELL_ARG)
          .help("Clear shell arguments"),
      )
      .arg(
        Arg::with_name(arg::UNSORTED)
          .long("unsorted")
          .short("u")
          .help("Return list and summary entries in source order"),
      )
      .arg(
        Arg::with_name(arg::VERBOSE)
          .short("v")
          .long("verbose")
          .multiple(true)
          .help("Use verbose output"),
      )
      .arg(
        Arg::with_name(arg::WORKING_DIRECTORY)
          .short("d")
          .long("working-directory")
          .takes_value(true)
          .help("Use <WORKING-DIRECTORY> as working directory. --justfile must also be set")
          .requires(arg::JUSTFILE),
      )
      .arg(Arg::with_name(cmd::CHOOSE).long("choose").help(CHOOSE_HELP))
      .arg(
        Arg::with_name(cmd::COMMAND)
          .long("command")
          .short("c")
          .min_values(1)
          .allow_hyphen_values(true)
          .help(
            "Run an arbitrary command with the working directory, `.env`, overrides, and exports \
             set",
          ),
      )
      .arg(
        Arg::with_name(cmd::COMPLETIONS)
          .long("completions")
          .takes_value(true)
          .value_name("SHELL")
          .possible_values(&clap::Shell::variants())
          .set(ArgSettings::CaseInsensitive)
          .help("Print shell completion script for <SHELL>"),
      )
      .arg(
        Arg::with_name(cmd::DUMP)
          .long("dump")
          .help("Print entire justfile"),
      )
      .arg(
        Arg::with_name(cmd::EDIT)
          .short("e")
          .long("edit")
          .help("Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`"),
      )
      .arg(Arg::with_name(cmd::EVALUATE).long("evaluate").help(
        "Evaluate and print all variables. If a variable name is given as an argument, only print \
         that variable's value.",
      ))
      .arg(
        Arg::with_name(cmd::INIT)
          .long("init")
          .help("Initialize new justfile in project root"),
      )
      .arg(
        Arg::with_name(cmd::LIST)
          .short("l")
          .long("list")
          .help("List available recipes and their arguments"),
      )
      .arg(
        Arg::with_name(cmd::SHOW)
          .short("s")
          .long("show")
          .takes_value(true)
          .value_name("RECIPE")
          .help("Show information about <RECIPE>"),
      )
      .arg(
        Arg::with_name(cmd::SUMMARY)
          .long("summary")
          .help("List names of available recipes"),
      )
      .arg(
        Arg::with_name(cmd::VARIABLES)
          .long("variables")
          .help("List names of variables"),
      )
      .group(ArgGroup::with_name("SUBCOMMAND").args(cmd::ALL))
      .arg(
        Arg::with_name(arg::ARGUMENTS)
          .multiple(true)
          .help("Overrides and recipe(s) to run, defaulting to the first recipe in the justfile"),
      );

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

  fn color_from_value(value: &str) -> ConfigResult<Color> {
    match value {
      arg::COLOR_AUTO => Ok(Color::auto()),
      arg::COLOR_ALWAYS => Ok(Color::always()),
      arg::COLOR_NEVER => Ok(Color::never()),
      _ => Err(ConfigError::Internal {
        message: format!("Invalid argument `{}` to --color.", value),
      }),
    }
  }

  pub(crate) fn from_matches(matches: &ArgMatches) -> ConfigResult<Self> {
    let invocation_directory = env::current_dir().context(config_error::CurrentDir)?;

    let verbosity = if matches.is_present(arg::QUIET) {
      Verbosity::Quiet
    } else {
      Verbosity::from_flag_occurrences(matches.occurrences_of(arg::VERBOSE))
    };

    let color = Self::color_from_value(
      matches
        .value_of(arg::COLOR)
        .expect("`--color` had no value"),
    )?;

    let set_count = matches.occurrences_of(arg::SET);
    let mut overrides = BTreeMap::new();
    if set_count > 0 {
      let mut values = matches.values_of(arg::SET).unwrap();
      for _ in 0..set_count {
        overrides.insert(
          values.next().unwrap().to_owned(),
          values.next().unwrap().to_owned(),
        );
      }
    }

    let positional = Positional::from_values(matches.values_of(arg::ARGUMENTS));

    for (name, value) in positional.overrides {
      overrides.insert(name.clone(), value.clone());
    }

    let search_config = {
      let justfile = matches.value_of(arg::JUSTFILE).map(PathBuf::from);
      let working_directory = matches.value_of(arg::WORKING_DIRECTORY).map(PathBuf::from);

      if let Some(search_directory) = positional.search_directory.map(PathBuf::from) {
        if justfile.is_some() || working_directory.is_some() {
          return Err(ConfigError::SearchDirConflict);
        }
        SearchConfig::FromSearchDirectory { search_directory }
      } else {
        match (justfile, working_directory) {
          (None, None) => SearchConfig::FromInvocationDirectory,
          (Some(justfile), None) => SearchConfig::WithJustfile { justfile },
          (Some(justfile), Some(working_directory)) =>
            SearchConfig::WithJustfileAndWorkingDirectory {
              justfile,
              working_directory,
            },
          (None, Some(_)) =>
            return Err(ConfigError::internal(
              "--working-directory set without --justfile",
            )),
        }
      }
    };

    for subcommand in cmd::ARGLESS {
      if matches.is_present(subcommand) {
        match (!overrides.is_empty(), !positional.arguments.is_empty()) {
          (false, false) => {},
          (true, false) => {
            return Err(ConfigError::SubcommandOverrides {
              subcommand,
              overrides,
            });
          },
          (false, true) => {
            return Err(ConfigError::SubcommandArguments {
              arguments: positional.arguments,
              subcommand,
            });
          },
          (true, true) => {
            return Err(ConfigError::SubcommandOverridesAndArguments {
              arguments: positional.arguments,
              subcommand,
              overrides,
            });
          },
        }
      }
    }

    let subcommand = if matches.is_present(cmd::CHOOSE) {
      Subcommand::Choose {
        chooser: matches.value_of(arg::CHOOSER).map(str::to_owned),
        overrides,
      }
    } else if let Some(values) = matches.values_of_os(cmd::COMMAND) {
      let mut arguments = values
        .into_iter()
        .map(OsStr::to_owned)
        .collect::<Vec<OsString>>();
      Subcommand::Command {
        binary: arguments.remove(0),
        arguments,
        overrides,
      }
    } else if let Some(shell) = matches.value_of(cmd::COMPLETIONS) {
      Subcommand::Completions {
        shell: shell.to_owned(),
      }
    } else if matches.is_present(cmd::EDIT) {
      Subcommand::Edit
    } else if matches.is_present(cmd::SUMMARY) {
      Subcommand::Summary
    } else if matches.is_present(cmd::DUMP) {
      Subcommand::Dump
    } else if matches.is_present(cmd::INIT) {
      Subcommand::Init
    } else if matches.is_present(cmd::LIST) {
      Subcommand::List
    } else if let Some(name) = matches.value_of(cmd::SHOW) {
      Subcommand::Show {
        name: name.to_owned(),
      }
    } else if matches.is_present(cmd::EVALUATE) {
      if positional.arguments.len() > 1 {
        return Err(ConfigError::SubcommandArguments {
          subcommand: cmd::EVALUATE,
          arguments:  positional
            .arguments
            .into_iter()
            .skip(1)
            .collect::<Vec<String>>(),
        });
      }

      Subcommand::Evaluate {
        variable: positional.arguments.into_iter().next(),
        overrides,
      }
    } else if matches.is_present(cmd::VARIABLES) {
      Subcommand::Variables
    } else {
      Subcommand::Run {
        arguments: positional.arguments,
        overrides,
      }
    };

    let shell_args = if matches.is_present(arg::CLEAR_SHELL_ARGS) {
      Vec::new()
    } else {
      matches
        .values_of(arg::SHELL_ARG)
        .unwrap()
        .map(str::to_owned)
        .collect()
    };

    let shell_present = matches.occurrences_of(arg::CLEAR_SHELL_ARGS) > 0
      || matches.occurrences_of(arg::SHELL) > 0
      || matches.occurrences_of(arg::SHELL_ARG) > 0;

    Ok(Self {
      dry_run: matches.is_present(arg::DRY_RUN),
      highlight: !matches.is_present(arg::NO_HIGHLIGHT),
      shell: matches.value_of(arg::SHELL).unwrap().to_owned(),
      load_dotenv: !matches.is_present(arg::NO_DOTENV),
      shell_command: matches.is_present(arg::SHELL_COMMAND),
      unsorted: matches.is_present(arg::UNSORTED),
      list_heading: matches
        .value_of(arg::LIST_HEADING)
        .unwrap_or("Available recipes:\n")
        .to_owned(),
      list_prefix: matches
        .value_of(arg::LIST_PREFIX)
        .unwrap_or("    ")
        .to_owned(),
      color,
      invocation_directory,
      search_config,
      shell_args,
      shell_present,
      subcommand,
      verbosity,
    })
  }

  pub(crate) fn run_subcommand(self) -> Result<(), i32> {
    use Subcommand::*;

    if self.subcommand == Init {
      return self.init();
    }

    if let Completions { shell } = self.subcommand {
      return Subcommand::completions(self.verbosity, &shell);
    }

    let search =
      Search::find(&self.search_config, &self.invocation_directory).eprint(self.color)?;

    if self.subcommand == Edit {
      return self.edit(&search);
    }

    let src = fs::read_to_string(&search.justfile)
      .map_err(|io_error| LoadError {
        io_error,
        path: &search.justfile,
      })
      .eprint(self.color)?;

    let justfile = Compiler::compile(&src).eprint(self.color)?;

    if self.verbosity.loud() {
      for warning in &justfile.warnings {
        if self.color.stderr().active() {
          eprintln!("{:#}", warning);
        } else {
          eprintln!("{}", warning);
        }
      }
    }

    match &self.subcommand {
      Choose { overrides, chooser } =>
        self.choose(justfile, &search, overrides, chooser.as_deref())?,
      Command { overrides, .. } => self.run(justfile, &search, overrides, &[])?,
      Dump => Self::dump(justfile),
      Evaluate { overrides, .. } => self.run(justfile, &search, overrides, &[])?,
      List => self.list(justfile),
      Run {
        arguments,
        overrides,
      } => self.run(justfile, &search, overrides, arguments)?,
      Show { ref name } => self.show(&name, justfile)?,
      Summary => self.summary(justfile),
      Variables => Self::variables(justfile),
      Completions { .. } | Edit | Init => unreachable!(),
    }

    Ok(())
  }

  fn choose(
    &self,
    justfile: Justfile,
    search: &Search,
    overrides: &BTreeMap<String, String>,
    chooser: Option<&str>,
  ) -> Result<(), i32> {
    let recipes = justfile
      .public_recipes(self.unsorted)
      .iter()
      .filter(|recipe| recipe.min_arguments() == 0)
      .cloned()
      .collect::<Vec<&Recipe<Dependency>>>();

    if recipes.is_empty() {
      if self.verbosity.loud() {
        eprintln!("Justfile contains no choosable recipes.");
      }
      return Err(EXIT_FAILURE);
    }

    let chooser = chooser
      .map(OsString::from)
      .or_else(|| env::var_os(CHOOSER_ENVIRONMENT_KEY))
      .unwrap_or_else(|| OsString::from(CHOOSER_DEFAULT));

    let result = justfile
      .settings
      .shell_command(self)
      .arg(&chooser)
      .current_dir(&search.working_directory)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn();

    let mut child = match result {
      Ok(child) => child,
      Err(error) => {
        if self.verbosity.loud() {
          eprintln!(
            "Chooser `{} {} {}` invocation failed: {}",
            justfile.settings.shell_binary(self),
            justfile.settings.shell_arguments(self).join(" "),
            chooser.to_string_lossy(),
            error
          );
        }
        return Err(EXIT_FAILURE);
      },
    };

    for recipe in recipes {
      if let Err(error) = child
        .stdin
        .as_mut()
        .expect("Child was created with piped stdio")
        .write_all(format!("{}\n", recipe.name).as_bytes())
      {
        if self.verbosity.loud() {
          eprintln!(
            "Failed to write to chooser `{}`: {}",
            chooser.to_string_lossy(),
            error
          );
        }
        return Err(EXIT_FAILURE);
      }
    }

    let output = match child.wait_with_output() {
      Ok(output) => output,
      Err(error) => {
        if self.verbosity.loud() {
          eprintln!(
            "Failed to read output from chooser `{}`: {}",
            chooser.to_string_lossy(),
            error
          );
        }
        return Err(EXIT_FAILURE);
      },
    };

    if !output.status.success() {
      if self.verbosity.loud() {
        eprintln!(
          "Chooser `{}` returned error: {}",
          chooser.to_string_lossy(),
          output.status
        );
      }
      return Err(output.status.code().unwrap_or(EXIT_FAILURE));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let recipes = stdout
      .trim()
      .split_whitespace()
      .map(str::to_owned)
      .collect::<Vec<String>>();

    self.run(justfile, search, overrides, &recipes)
  }

  fn dump(justfile: Justfile) {
    println!("{}", justfile);
  }

  pub(crate) fn edit(&self, search: &Search) -> Result<(), i32> {
    let editor = env::var_os("VISUAL")
      .or_else(|| env::var_os("EDITOR"))
      .unwrap_or_else(|| "vim".into());

    let error = Command::new(&editor)
      .current_dir(&search.working_directory)
      .arg(&search.justfile)
      .status();

    match error {
      Ok(status) =>
        if status.success() {
          Ok(())
        } else {
          if self.verbosity.loud() {
            eprintln!("Editor `{}` failed: {}", editor.to_string_lossy(), status);
          }
          Err(status.code().unwrap_or(EXIT_FAILURE))
        },
      Err(error) => {
        if self.verbosity.loud() {
          eprintln!(
            "Editor `{}` invocation failed: {}",
            editor.to_string_lossy(),
            error
          );
        }
        Err(EXIT_FAILURE)
      },
    }
  }

  pub(crate) fn init(&self) -> Result<(), i32> {
    let search =
      Search::init(&self.search_config, &self.invocation_directory).eprint(self.color)?;

    if search.justfile.exists() {
      if self.verbosity.loud() {
        eprintln!("Justfile `{}` already exists", search.justfile.display());
      }
      Err(EXIT_FAILURE)
    } else if let Err(err) = fs::write(&search.justfile, INIT_JUSTFILE) {
      if self.verbosity.loud() {
        eprintln!(
          "Failed to write justfile to `{}`: {}",
          search.justfile.display(),
          err
        );
      }
      Err(EXIT_FAILURE)
    } else {
      if self.verbosity.loud() {
        eprintln!("Wrote justfile to `{}`", search.justfile.display());
      }
      Ok(())
    }
  }

  fn list(&self, justfile: Justfile) {
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
          line_width += UnicodeWidthStr::width(format!(" {}", parameter).as_str());
        }

        if line_width <= 30 {
          line_widths.insert(name, line_width);
        }
      }
    }

    let max_line_width = cmp::min(line_widths.values().cloned().max().unwrap_or(0), 30);

    let doc_color = self.color.stdout().doc();
    print!("{}", self.list_heading);

    for recipe in justfile.public_recipes(self.unsorted) {
      let name = recipe.name();

      for (i, name) in iter::once(&name)
        .chain(recipe_aliases.get(name).unwrap_or(&Vec::new()))
        .enumerate()
      {
        print!("{}{}", self.list_prefix, name);
        for parameter in &recipe.parameters {
          if self.color.stdout().active() {
            print!(" {:#}", parameter);
          } else {
            print!(" {}", parameter);
          }
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
          },
        }
        println!();
      }
    }
  }

  fn run(
    &self,
    justfile: Justfile,
    search: &Search,
    overrides: &BTreeMap<String, String>,
    arguments: &[String],
  ) -> Result<(), i32> {
    if let Err(error) = InterruptHandler::install(self.verbosity) {
      warn!("Failed to set CTRL-C handler: {}", error);
    }

    let result = justfile.run(&self, search, overrides, arguments);

    if !self.verbosity.quiet() {
      result.eprint(self.color)
    } else {
      result.map_err(|err| err.code())
    }
  }

  fn show(&self, name: &str, justfile: Justfile) -> Result<(), i32> {
    if let Some(alias) = justfile.get_alias(name) {
      let recipe = justfile.get_recipe(alias.target.name.lexeme()).unwrap();
      println!("{}", alias);
      println!("{}", recipe);
      Ok(())
    } else if let Some(recipe) = justfile.get_recipe(name) {
      println!("{}", recipe);
      Ok(())
    } else {
      if self.verbosity.loud() {
        eprintln!("Justfile does not contain recipe `{}`.", name);
        if let Some(suggestion) = justfile.suggest_recipe(name) {
          eprintln!("{}", suggestion);
        }
      }
      Err(EXIT_FAILURE)
    }
  }

  fn summary(&self, justfile: Justfile) {
    if justfile.count() == 0 {
      if self.verbosity.loud() {
        eprintln!("Justfile contains no recipes.");
      }
    } else {
      let summary = justfile
        .public_recipes(self.unsorted)
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

  use pretty_assertions::assert_eq;

  // This test guards against unintended changes to the argument parser. We should
  // have proper tests for all the flags, but this will do for now.
  #[test]
  fn help() {
    const EXPECTED_HELP: &str = "just v0.9.2
Casey Rodarmor <casey@rodarmor.com>
🤖 Just a command runner \
                                 - https://github.com/casey/just

USAGE:
    just [FLAGS] [OPTIONS] [--] [ARGUMENTS]...

FLAGS:
        --choose              Select one or more recipes to run using a binary. If `--chooser` is \
                                 not passed the chooser
                              defaults to the value of $JUST_CHOOSER, falling back to `fzf`
        --clear-shell-args    Clear shell arguments
        --dry-run             Print what just would do without doing it
        --dump                Print entire justfile
    -e, --edit                Edit justfile with editor given by $VISUAL or $EDITOR, falling back \
                                 to `vim`
        --evaluate            Evaluate and print all variables. If a variable name is given as an \
                                 argument, only print
                              that variable's value.
        --highlight           Highlight echoed recipe lines in bold
        --init                Initialize new justfile in project root
    -l, --list                List available recipes and their arguments
        --no-dotenv           Don't load `.env` file
        --no-highlight        Don't highlight echoed recipe lines in bold
    -q, --quiet               Suppress all output
        --shell-command       Invoke <COMMAND> with the shell used to run recipe lines and \
                                 backticks
        --summary             List names of available recipes
    -u, --unsorted            Return list and summary entries in source order
        --variables           List names of variables
    -v, --verbose             Use verbose output

OPTIONS:
        --chooser <CHOOSER>                        Override binary invoked by `--choose`
        --color <COLOR>
            Print colorful output [default: auto]  [possible values: auto, always, never]

    -c, --command <COMMAND>
            Run an arbitrary command with the working directory, `.env`, overrides, and exports set

        --completions <SHELL>
            Print shell completion script for <SHELL> [possible values: zsh, bash, fish, \
                                 powershell, elvish]

    -f, --justfile <JUSTFILE>                      Use <JUSTFILE> as justfile
        --list-heading <TEXT>                      Print <TEXT> before list
        --list-prefix <TEXT>                       Print <TEXT> before each list item
        --set <VARIABLE> <VALUE>                   Override <VARIABLE> with <VALUE>
        --shell <SHELL>                            Invoke <SHELL> to run recipes [default: sh]
        --shell-arg <SHELL-ARG>...                 Invoke shell with <SHELL-ARG> as an argument \
                                 [default: -cu]
    -s, --show <RECIPE>                            Show information about <RECIPE>
    -d, --working-directory <WORKING-DIRECTORY>
            Use <WORKING-DIRECTORY> as working directory. --justfile must also be set


ARGS:
    <ARGUMENTS>...    Overrides and recipe(s) to run, defaulting to the first recipe in the \
                                 justfile";

    let app = Config::app().setting(AppSettings::ColorNever);
    let mut buffer = Vec::new();
    app.write_help(&mut buffer).unwrap();
    let help = str::from_utf8(&buffer).unwrap();

    assert_eq!(help, EXPECTED_HELP);
  }

  macro_rules! test {
    {
      name: $name:ident,
      args: [$($arg:expr),*],
      $(color: $color:expr,)?
      $(dry_run: $dry_run:expr,)?
      $(highlight: $highlight:expr,)?
      $(search_config: $search_config:expr,)?
      $(shell: $shell:expr,)?
      $(shell_args: $shell_args:expr,)?
      $(shell_present: $shell_present:expr,)?
      $(subcommand: $subcommand:expr,)?
      $(unsorted: $unsorted:expr,)?
      $(verbosity: $verbosity:expr,)?
    } => {
      #[test]
      fn $name() {
        let arguments = &[
          "just",
          $($arg,)*
        ];

        let want = Config {
          $(color: $color,)?
          $(dry_run: $dry_run,)?
          $(highlight: $highlight,)?
          $(search_config: $search_config,)?
          $(shell: $shell.to_owned(),)?
          $(shell_args: $shell_args,)?
          $(shell_present: $shell_present,)?
          $(subcommand: $subcommand,)?
          $(unsorted: $unsorted,)?
          $(verbosity: $verbosity,)?
          ..testing::config(&[])
        };

        test(arguments, want);
      }
    }
  }

  fn test(arguments: &[&str], want: Config) {
    let app = Config::app();
    let matches = app
      .get_matches_from_safe(arguments)
      .expect("agument parsing failed");
    let have = Config::from_matches(&matches).expect("config parsing failed");
    assert_eq!(have, want);
  }

  macro_rules! error {
    {
      name: $name:ident,
      args: [$($arg:expr),*],
    } => {
      #[test]
      fn $name() {
        let arguments = &[
          "just",
          $($arg,)*
        ];

        let app = Config::app();

        app.get_matches_from_safe(arguments).expect_err("Expected clap error");
      }
    };
    {
      name: $name:ident,
      args: [$($arg:expr),*],
      error: $error:pat,
      $(check: $check:block,)?
    } => {
      #[test]
      fn $name() {
        let arguments = &[
          "just",
          $($arg,)*
        ];

        let app = Config::app();

        let matches = app.get_matches_from_safe(arguments).expect("Matching failes");

        match Config::from_matches(&matches).expect_err("config parsing succeeded") {
          $error => { $($check)? }
          other => panic!("Unexpected config error: {}", other),
        }
      }
    }
  }

  macro_rules! map {
    {} => {
      BTreeMap::new()
    };
    {
      $($key:literal : $value:literal),* $(,)?
    } => {
      {
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        $(
          map.insert($key.to_owned(), $value.to_owned());
        )*
        map
      }
    }
  }

  test! {
    name: default_config,
    args: [],
  }

  test! {
    name: color_default,
    args: [],
    color: Color::auto(),
  }

  test! {
    name: color_never,
    args: ["--color", "never"],
    color: Color::never(),
  }

  test! {
    name: color_always,
    args: ["--color", "always"],
    color: Color::always(),
  }

  test! {
    name: color_auto,
    args: ["--color", "auto"],
    color: Color::auto(),
  }

  error! {
    name: color_bad_value,
    args: ["--color", "foo"],
  }

  test! {
    name: dry_run_default,
    args: [],
    dry_run: false,
  }

  test! {
    name: dry_run_true,
    args: ["--dry-run"],
    dry_run: true,
  }

  error! {
    name: dry_run_quiet,
    args: ["--dry-run", "--quiet"],
  }

  test! {
    name: highlight_default,
    args: [],
    highlight: true,
  }

  test! {
    name: highlight_yes,
    args: ["--highlight"],
    highlight: true,
  }

  test! {
    name: highlight_no,
    args: ["--no-highlight"],
    highlight: false,
  }

  test! {
    name: highlight_no_yes,
    args: ["--no-highlight", "--highlight"],
    highlight: true,
  }

  test! {
    name: highlight_no_yes_no,
    args: ["--no-highlight", "--highlight", "--no-highlight"],
    highlight: false,
  }

  test! {
    name: highlight_yes_no,
    args: ["--highlight", "--no-highlight"],
    highlight: false,
  }

  test! {
    name: unsorted_default,
    args: [],
    unsorted: false,
  }

  test! {
    name: unsorted_long,
    args: ["--unsorted"],
    unsorted: true,
  }

  test! {
    name: unsorted_short,
    args: ["-u"],
    unsorted: true,
  }

  test! {
    name: quiet_default,
    args: [],
    verbosity: Verbosity::Taciturn,
  }

  test! {
    name: quiet_long,
    args: ["--quiet"],
    verbosity: Verbosity::Quiet,
  }

  test! {
    name: quiet_short,
    args: ["-q"],
    verbosity: Verbosity::Quiet,
  }

  test! {
    name: set_default,
    args: [],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!(),
    },
  }

  test! {
    name: set_one,
    args: ["--set", "foo", "bar"],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!{"foo": "bar"},
    },
  }

  test! {
    name: set_empty,
    args: ["--set", "foo", ""],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!{"foo": ""},
    },
  }

  test! {
    name: set_two,
    args: ["--set", "foo", "bar", "--set", "bar", "baz"],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!{"foo": "bar", "bar": "baz"},
    },
  }

  test! {
    name: set_override,
    args: ["--set", "foo", "bar", "--set", "foo", "baz"],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!{"foo": "baz"},
    },
  }

  error! {
    name: set_bad,
    args: ["--set", "foo"],
  }

  test! {
    name: shell_default,
    args: [],
    shell: "sh",
    shell_args: vec!["-cu".to_owned()],
    shell_present: false,
  }

  test! {
    name: shell_set,
    args: ["--shell", "tclsh"],
    shell: "tclsh",
    shell_present: true,
  }

  test! {
    name: verbosity_default,
    args: [],
    verbosity: Verbosity::Taciturn,
  }

  test! {
    name: verbosity_long,
    args: ["--verbose"],
    verbosity: Verbosity::Loquacious,
  }

  test! {
    name: verbosity_loquacious,
    args: ["-v"],
    verbosity: Verbosity::Loquacious,
  }

  test! {
    name: verbosity_grandiloquent,
    args: ["-v", "-v"],
    verbosity: Verbosity::Grandiloquent,
  }

  test! {
    name: verbosity_great_grandiloquent,
    args: ["-v", "-v", "-v"],
    verbosity: Verbosity::Grandiloquent,
  }

  test! {
    name: subcommand_default,
    args: [],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!{},
    },
  }

  error! {
    name: subcommand_conflict_summary,
    args: ["--list", "--summary"],
  }

  error! {
    name: subcommand_conflict_dump,
    args: ["--list", "--dump"],
  }

  error! {
    name: subcommand_conflict_init,
    args: ["--list", "--init"],
  }

  error! {
    name: subcommand_conflict_evaluate,
    args: ["--list", "--evaluate"],
  }

  error! {
    name: subcommand_conflict_show,
    args: ["--list", "--show"],
  }

  error! {
    name: subcommand_conflict_completions,
    args: ["--list", "--completions"],
  }

  error! {
    name: subcommand_conflict_variables,
    args: ["--list", "--variables"],
  }

  error! {
    name: subcommand_conflict_choose,
    args: ["--list", "--choose"],
  }

  test! {
    name: subcommand_completions,
    args: ["--completions", "bash"],
    subcommand: Subcommand::Completions{shell: "bash".to_owned()},
  }

  test! {
    name: subcommand_completions_uppercase,
    args: ["--completions", "BASH"],
    subcommand: Subcommand::Completions{shell: "BASH".to_owned()},
  }

  error! {
    name: subcommand_completions_invalid,
    args: ["--completions", "monstersh"],
  }

  test! {
    name: subcommand_dump,
    args: ["--dump"],
    subcommand: Subcommand::Dump,
  }

  test! {
    name: subcommand_edit,
    args: ["--edit"],
    subcommand: Subcommand::Edit,
  }

  test! {
    name: subcommand_evaluate,
    args: ["--evaluate"],
    subcommand: Subcommand::Evaluate {
      overrides: map!{},
      variable: None,
    },
  }

  test! {
    name: subcommand_evaluate_overrides,
    args: ["--evaluate", "x=y"],
    subcommand: Subcommand::Evaluate {
      overrides: map!{"x": "y"},
      variable: None,
    },
  }

  test! {
    name: subcommand_evaluate_overrides_with_argument,
    args: ["--evaluate", "x=y", "foo"],
    subcommand: Subcommand::Evaluate {
      overrides: map!{"x": "y"},
      variable: Some("foo".to_owned()),
    },
  }

  test! {
    name: subcommand_list_long,
    args: ["--list"],
    subcommand: Subcommand::List,
  }

  test! {
    name: subcommand_list_short,
    args: ["-l"],
    subcommand: Subcommand::List,
  }

  test! {
    name: subcommand_show_long,
    args: ["--show", "build"],
    subcommand: Subcommand::Show { name: String::from("build") },
  }

  test! {
    name: subcommand_show_short,
    args: ["-s", "build"],
    subcommand: Subcommand::Show { name: String::from("build") },
  }

  error! {
    name: subcommand_show_no_arg,
    args: ["--show"],
  }

  test! {
    name: subcommand_summary,
    args: ["--summary"],
    subcommand: Subcommand::Summary,
  }

  test! {
    name: arguments,
    args: ["foo", "bar"],
    subcommand: Subcommand::Run {
      arguments: vec![String::from("foo"), String::from("bar")],
      overrides: map!{},
    },
  }

  test! {
    name: arguments_leading_equals,
    args: ["=foo"],
    subcommand: Subcommand::Run {
      arguments: vec!["=foo".to_owned()],
      overrides: map!{},
    },
  }

  test! {
    name: overrides,
    args: ["foo=bar", "bar=baz"],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!{"foo": "bar", "bar": "baz"},
    },
  }

  test! {
    name: overrides_empty,
    args: ["foo=", "bar="],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!{"foo": "", "bar": ""},
    },
  }

  test! {
    name: overrides_override_sets,
    args: ["--set", "foo", "0", "--set", "bar", "1", "foo=bar", "bar=baz"],
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
      overrides: map!{"foo": "bar", "bar": "baz"},
    },
  }

  test! {
    name: shell_args_default,
    args: [],
    shell_args: vec!["-cu".to_owned()],
  }

  test! {
    name: shell_args_set_hyphen,
    args: ["--shell-arg", "--foo"],
    shell_args: vec!["--foo".to_owned()],
    shell_present: true,
  }

  test! {
    name: shell_args_set_word,
    args: ["--shell-arg", "foo"],
    shell_args: vec!["foo".to_owned()],
    shell_present: true,
  }

  test! {
    name: shell_args_set_multiple,
    args: ["--shell-arg", "foo", "--shell-arg", "bar"],
    shell_args: vec!["foo".to_owned(), "bar".to_owned()],
    shell_present: true,
  }

  test! {
    name: shell_args_clear,
    args: ["--clear-shell-args"],
    shell_args: vec![],
    shell_present: true,
  }

  test! {
    name: shell_args_clear_and_set,
    args: ["--clear-shell-args", "--shell-arg", "bar"],
    shell_args: vec!["bar".to_owned()],
    shell_present: true,
  }

  test! {
    name: shell_args_set_and_clear,
    args: ["--shell-arg", "bar", "--clear-shell-args"],
    shell_args: vec![],
    shell_present: true,
  }

  test! {
    name: shell_args_set_multiple_and_clear,
    args: ["--shell-arg", "bar", "--shell-arg", "baz", "--clear-shell-args"],
    shell_args: vec![],
    shell_present: true,
  }

  test! {
    name: search_config_default,
    args: [],
    search_config: SearchConfig::FromInvocationDirectory,
  }

  test! {
    name: search_config_from_working_directory_and_justfile,
    args: ["--working-directory", "foo", "--justfile", "bar"],
    search_config: SearchConfig::WithJustfileAndWorkingDirectory {
      justfile: PathBuf::from("bar"),
      working_directory: PathBuf::from("foo"),
    },
  }

  test! {
    name: search_config_justfile_long,
    args: ["--justfile", "foo"],
    search_config: SearchConfig::WithJustfile {
      justfile: PathBuf::from("foo"),
    },
  }

  test! {
    name: search_config_justfile_short,
    args: ["-f", "foo"],
    search_config: SearchConfig::WithJustfile {
      justfile: PathBuf::from("foo"),
    },
  }

  test! {
    name: search_directory_parent,
    args: ["../"],
    search_config: SearchConfig::FromSearchDirectory {
      search_directory: PathBuf::from(".."),
    },
  }

  test! {
    name: search_directory_parent_with_recipe,
    args: ["../build"],
    search_config: SearchConfig::FromSearchDirectory {
      search_directory: PathBuf::from(".."),
    },
    subcommand: Subcommand::Run { arguments: vec!["build".to_owned()], overrides: BTreeMap::new() },
  }

  test! {
    name: search_directory_child,
    args: ["foo/"],
    search_config: SearchConfig::FromSearchDirectory {
      search_directory: PathBuf::from("foo"),
    },
  }

  test! {
    name: search_directory_deep,
    args: ["foo/bar/"],
    search_config: SearchConfig::FromSearchDirectory {
      search_directory: PathBuf::from("foo/bar"),
    },
  }

  test! {
    name: search_directory_child_with_recipe,
    args: ["foo/build"],
    search_config: SearchConfig::FromSearchDirectory {
      search_directory: PathBuf::from("foo"),
    },
    subcommand: Subcommand::Run { arguments: vec!["build".to_owned()], overrides: BTreeMap::new() },
  }

  error! {
    name: search_directory_conflict_justfile,
    args: ["--justfile", "bar", "foo/build"],
    error: ConfigError::SearchDirConflict,
  }

  error! {
    name: search_directory_conflict_working_directory,
    args: ["--justfile", "bar", "--working-directory", "baz", "foo/build"],
    error: ConfigError::SearchDirConflict,
  }

  error! {
    name: completions_arguments,
    args: ["--completions", "zsh", "foo"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::COMPLETIONS);
      assert_eq!(arguments, &["foo"]);
    },
  }

  error! {
    name: list_arguments,
    args: ["--list", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::LIST);
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: dump_arguments,
    args: ["--dump", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::DUMP);
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: edit_arguments,
    args: ["--edit", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::EDIT);
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: init_arguments,
    args: ["--init", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::INIT);
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: show_arguments,
    args: ["--show", "foo", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::SHOW);
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: summary_arguments,
    args: ["--summary", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::SUMMARY);
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: subcommand_overrides_and_arguments,
    args: ["--summary", "bar=baz", "bar"],
    error: ConfigError::SubcommandOverridesAndArguments { subcommand, arguments, overrides },
    check: {
      assert_eq!(subcommand, cmd::SUMMARY);
      assert_eq!(overrides, map!{"bar": "baz"});
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: summary_overrides,
    args: ["--summary", "bar=baz"],
    error: ConfigError::SubcommandOverrides { subcommand, overrides },
    check: {
      assert_eq!(subcommand, cmd::SUMMARY);
      assert_eq!(overrides, map!{"bar": "baz"});
    },
  }

  #[test]
  fn init_justfile() {
    testing::compile(INIT_JUSTFILE);
  }
}
