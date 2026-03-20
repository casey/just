use {
  super::*,
  clap::{
    ArgAction, ArgGroup, Parser,
    builder::{
      FalseyValueParser, Styles,
      styling::{AnsiColor, Effects},
    },
  },
};

const HEADING: &str = "Commands";

const ABOUT: &str = concat!(
  env!("CARGO_PKG_DESCRIPTION"),
  " - ",
  env!("CARGO_PKG_HOMEPAGE")
);

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, Parser)]
#[command(
  name = env!("CARGO_PKG_NAME"),
  bin_name = env!("CARGO_PKG_NAME"),
  version = env!("CARGO_PKG_VERSION"),
  author = env!("CARGO_PKG_AUTHORS"),
  about = ABOUT,
  trailing_var_arg = true,
  styles = Arguments::styles(),
  group = ArgGroup::new("SUBCOMMAND").args([
    "changelog", "choose", "COMMAND", "COMPLETIONS", "dump", "edit",
    "evaluate", "FORMAT", "GROUPS", "init", "json", "LIST", "man",
    "REQUEST", "SHOW", "summary", "USAGE", "variables",
  ]),
)]
pub(crate) struct Arguments {
  #[arg(
    long = "alias-style",
    env = "JUST_ALIAS_STYLE",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(AliasStyle),
    default_value = "right",
    help = "Set list command alias display style",
    conflicts_with = "no_aliases",
  )]
  alias_style: AliasStyle,

  #[arg(
    long = "allow-missing",
    env = "JUST_ALLOW_MISSING",
    action = ArgAction::SetTrue,
    help = "Ignore missing recipe and module errors",
  )]
  allow_missing: bool,

  #[arg(
    long = "ceiling",
    env = "JUST_CEILING",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(PathBuf),
    help = "Do not ascend above <CEILING> directory when searching for a justfile.",
  )]
  ceiling: Option<PathBuf>,

  #[arg(
    long = "check",
    action = ArgAction::SetTrue,
    requires = "FORMAT",
    help = "Run `--fmt` in 'check' mode. Exits with 0 if justfile is formatted correctly. \
            Exits with 1 and prints a diff if formatting is required.",
  )]
  check: bool,

  #[arg(
    long = "chooser",
    env = "JUST_CHOOSER",
    action = ArgAction::Set,
    help = "Override binary invoked by `--choose`",
  )]
  chooser: Option<String>,

  #[arg(
    long = "clear-shell-args",
    action = ArgAction::SetTrue,
    overrides_with = "shell_arg",
    help = "Clear shell arguments",
  )]
  clear_shell_args: bool,

  #[arg(
    long = "color",
    env = "JUST_COLOR",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(UseColor),
    default_value = "auto",
    help = "Print colorful output",
  )]
  color: UseColor,

  #[arg(
    long = "command-color",
    env = "JUST_COMMAND_COLOR",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(CommandColor),
    help = "Echo recipe lines in <COMMAND-COLOR>",
  )]
  command_color: Option<CommandColor>,

  #[arg(
    long = "cygpath",
    env = "JUST_CYGPATH",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(PathBuf),
    default_value = "cygpath",
    help = "Use binary at <CYGPATH> to convert between unix and Windows paths.",
  )]
  cygpath: PathBuf,

  #[arg(
    long = "dotenv-filename",
    action = ArgAction::Set,
    help = "Search for environment file named <DOTENV-FILENAME> instead of `.env`",
    conflicts_with = "dotenv_path",
  )]
  dotenv_filename: Option<String>,

  #[arg(
    short = 'E',
    long = "dotenv-path",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(PathBuf),
    help = "Load <DOTENV-PATH> as environment file instead of searching for one",
  )]
  dotenv_path: Option<PathBuf>,

  #[arg(
    short = 'n',
    long = "dry-run",
    env = "JUST_DRY_RUN",
    action = ArgAction::SetTrue,
    help = "Print what just would do without doing it",
    conflicts_with = "quiet",
  )]
  dry_run: bool,

  #[arg(
    long = "dump-format",
    env = "JUST_DUMP_FORMAT",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(DumpFormat),
    default_value = "just",
    value_name = "FORMAT",
    help = "Dump justfile as <FORMAT>",
  )]
  dump_format: DumpFormat,

  #[arg(
    long = "explain",
    env = "JUST_EXPLAIN",
    action = ArgAction::SetTrue,
    help = "Print recipe doc comment before running it",
  )]
  explain: bool,

  #[arg(
    long = "global-justfile",
    short = 'g',
    action = ArgAction::SetTrue,
    conflicts_with = "justfile",
    conflicts_with = "working_directory",
    help = "Use global justfile",
  )]
  global_justfile: bool,

  #[arg(
    long = "group",
    env = "JUST_GROUP",
    action = ArgAction::Append,
    help = "Only list recipes in <GROUP>",
    requires = "LIST",
  )]
  group: Vec<String>,

  #[arg(
    long = "highlight",
    env = "JUST_HIGHLIGHT",
    action = ArgAction::SetTrue,
    help = "Highlight echoed recipe lines in bold",
    overrides_with = "no_highlight",
  )]
  highlight: bool,

  #[arg(
    short = 'f',
    long = "justfile",
    env = "JUST_JUSTFILE",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(PathBuf),
    help = "Use <JUSTFILE> as justfile",
  )]
  justfile: Option<PathBuf>,

  #[arg(
    long = "list-heading",
    env = "JUST_LIST_HEADING",
    help = "Print <TEXT> before list",
    value_name = "TEXT",
    default_value = "Available recipes:\n",
    action = ArgAction::Set,
  )]
  list_heading: String,

  #[arg(
    long = "list-prefix",
    env = "JUST_LIST_PREFIX",
    help = "Print <TEXT> before each list item",
    value_name = "TEXT",
    default_value = "    ",
    action = ArgAction::Set,
  )]
  list_prefix: String,

  #[arg(
    long = "list-submodules",
    env = "JUST_LIST_SUBMODULES",
    help = "List recipes in submodules",
    action = ArgAction::SetTrue,
    requires = "LIST",
  )]
  list_submodules: bool,

  #[arg(
    long = "no-aliases",
    env = "JUST_NO_ALIASES",
    action = ArgAction::SetTrue,
    help = "Don't show aliases in list",
  )]
  no_aliases: bool,

  #[arg(
    long = "no-deps",
    env = "JUST_NO_DEPS",
    alias = "no-dependencies",
    action = ArgAction::SetTrue,
    help = "Don't run recipe dependencies",
  )]
  no_deps: bool,

  #[arg(
    long = "no-dotenv",
    env = "JUST_NO_DOTENV",
    action = ArgAction::SetTrue,
    help = "Don't load `.env` file",
  )]
  no_dotenv: bool,

  #[arg(
    long = "no-highlight",
    env = "JUST_NO_HIGHLIGHT",
    action = ArgAction::SetTrue,
    help = "Don't highlight echoed recipe lines in bold",
    overrides_with = "highlight",
  )]
  no_highlight: bool,

  #[arg(
    long = "one",
    env = "JUST_ONE",
    action = ArgAction::SetTrue,
    help = "Forbid multiple recipes from being invoked on the command line",
  )]
  one: bool,

  #[arg(
    short = 'q',
    long = "quiet",
    env = "JUST_QUIET",
    action = ArgAction::SetTrue,
    help = "Suppress all output",
    conflicts_with = "dry_run",
  )]
  quiet: bool,

  #[arg(
    long = "set",
    action = ArgAction::Append,
    num_args = 2,
    value_names = ["VARIABLE", "VALUE"],
    help = "Override <VARIABLE> with <VALUE>",
  )]
  set: Vec<String>,

  #[arg(
    long = "shell",
    action = ArgAction::Set,
    help = "Invoke <SHELL> to run recipes",
  )]
  shell: Option<String>,

  #[arg(
    long = "shell-arg",
    action = ArgAction::Append,
    allow_hyphen_values = true,
    overrides_with = "clear_shell_args",
    help = "Invoke shell with <SHELL-ARG> as an argument",
  )]
  shell_arg: Vec<String>,

  #[arg(
    long = "shell-command",
    requires = "COMMAND",
    action = ArgAction::SetTrue,
    help = "Invoke <COMMAND> with the shell used to run recipe lines and backticks",
  )]
  shell_command: bool,

  #[arg(
    long = "tempdir",
    env = "JUST_TEMPDIR",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(PathBuf),
    help = "Save temporary files to <TEMPDIR>.",
  )]
  tempdir: Option<PathBuf>,

  #[arg(
    long = "timestamp",
    env = "JUST_TIMESTAMP",
    action = ArgAction::SetTrue,
    help = "Print recipe command timestamps",
  )]
  timestamp: bool,

  #[arg(
    long = "timestamp-format",
    env = "JUST_TIMESTAMP_FORMAT",
    action = ArgAction::Set,
    default_value = "%H:%M:%S",
    help = "Timestamp format string",
  )]
  timestamp_format: String,

  #[arg(
    long = "unsorted",
    env = "JUST_UNSORTED",
    short = 'u',
    action = ArgAction::SetTrue,
    help = "Return list and summary entries in source order",
  )]
  unsorted: bool,

  #[arg(
    long = "unstable",
    env = "JUST_UNSTABLE",
    action = ArgAction::SetTrue,
    value_parser = FalseyValueParser::new(),
    help = "Enable unstable features",
  )]
  unstable: bool,

  #[arg(
    short = 'v',
    long = "verbose",
    env = "JUST_VERBOSE",
    action = ArgAction::Count,
    help = "Use verbose output",
  )]
  verbose: u8,

  #[arg(
    short = 'd',
    long = "working-directory",
    env = "JUST_WORKING_DIRECTORY",
    action = ArgAction::Set,
    value_parser = clap::value_parser!(PathBuf),
    help = "Use <WORKING-DIRECTORY> as working directory. --justfile must also be set",
    requires = "justfile",
  )]
  working_directory: Option<PathBuf>,

  #[arg(
    long = "yes",
    env = "JUST_YES",
    action = ArgAction::SetTrue,
    help = "Automatically confirm all recipes.",
  )]
  yes: bool,

  #[arg(
    long = "changelog",
    action = ArgAction::SetTrue,
    help = "Print changelog",
    help_heading = HEADING,
  )]
  changelog: bool,

  #[arg(
    long = "choose",
    action = ArgAction::SetTrue,
    help = "Select one or more recipes to run using a binary chooser. If `--chooser` is not \
            passed the chooser defaults to the value of $JUST_CHOOSER, falling back to `fzf`",
    help_heading = HEADING,
  )]
  choose: bool,

  #[arg(
    id = "COMMAND",
    long = "command",
    short = 'c',
    num_args = 1..,
    allow_hyphen_values = true,
    action = ArgAction::Append,
    value_parser = clap::value_parser!(OsString),
    help = "Run an arbitrary command with the working directory, `.env`, overrides, and exports \
            set",
    help_heading = HEADING,
  )]
  command: Vec<OsString>,

  #[arg(
    id = "COMPLETIONS",
    long = "completions",
    action = ArgAction::Set,
    value_name = "SHELL",
    value_parser = clap::value_parser!(completions::Shell),
    ignore_case = true,
    help = "Print shell completion script for <SHELL>",
    help_heading = HEADING,
  )]
  completions: Option<completions::Shell>,

  #[arg(
    long = "dump",
    action = ArgAction::SetTrue,
    help = "Print justfile",
    help_heading = HEADING,
  )]
  dump: bool,

  #[arg(
    short = 'e',
    long = "edit",
    action = ArgAction::SetTrue,
    help = "Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`",
    help_heading = HEADING,
  )]
  edit: bool,

  #[arg(
    long = "evaluate",
    alias = "eval",
    action = ArgAction::SetTrue,
    help = "Evaluate and print all variables. If a variable name is given as an argument, only \
            print that variable's value.",
    help_heading = HEADING,
  )]
  evaluate: bool,

  #[arg(
    id = "FORMAT",
    long = "fmt",
    alias = "format",
    action = ArgAction::SetTrue,
    help = "Format and overwrite justfile",
    help_heading = HEADING,
  )]
  format: bool,

  #[arg(
    id = "GROUPS",
    long = "groups",
    action = ArgAction::SetTrue,
    help = "List recipe groups",
    help_heading = HEADING,
  )]
  groups_cmd: bool,

  #[arg(
    long = "init",
    alias = "initialize",
    action = ArgAction::SetTrue,
    help = "Initialize new justfile in project root",
    help_heading = HEADING,
  )]
  init: bool,

  #[arg(
    long = "json",
    action = ArgAction::SetTrue,
    conflicts_with = "dump_format",
    help = "Print justfile as JSON",
    help_heading = HEADING,
  )]
  json: bool,

  #[arg(
    id = "LIST",
    short = 'l',
    long = "list",
    num_args = 0..,
    value_name = "MODULE",
    action = ArgAction::Set,
    conflicts_with = "ARGUMENTS",
    help = "List available recipes in <MODULE> or root if omitted",
    help_heading = HEADING,
  )]
  list: Option<Vec<String>>,

  #[arg(
    long = "man",
    action = ArgAction::SetTrue,
    help = "Print man page",
    help_heading = HEADING,
  )]
  man: bool,

  #[arg(
    id = "REQUEST",
    long = "request",
    action = ArgAction::Set,
    hide = true,
    help = "Execute <REQUEST>. For internal testing purposes only. May be changed or removed at \
            any time.",
    help_heading = "REQUEST",
  )]
  request: Option<String>,

  #[arg(
    id = "SHOW",
    short = 's',
    long = "show",
    num_args = 1..,
    action = ArgAction::Set,
    value_name = "PATH",
    conflicts_with = "ARGUMENTS",
    help = "Show recipe at <PATH>",
    help_heading = HEADING,
  )]
  show: Option<Vec<String>>,

  #[arg(
    long = "summary",
    action = ArgAction::SetTrue,
    help = "List names of available recipes",
    help_heading = HEADING,
  )]
  summary: bool,

  #[arg(
    id = "USAGE",
    long = "usage",
    num_args = 1..,
    value_name = "PATH",
    action = ArgAction::Set,
    conflicts_with = "ARGUMENTS",
    help = "Print recipe usage information",
    help_heading = HEADING,
  )]
  usage: Option<Vec<String>>,

  #[arg(
    long = "variables",
    action = ArgAction::SetTrue,
    help = "List names of variables",
    help_heading = HEADING,
  )]
  variables: bool,

  #[arg(
    id = "ARGUMENTS",
    num_args = 1..,
    action = ArgAction::Append,
    help = "Overrides and recipe(s) to run, defaulting to the first recipe in the justfile",
  )]
  arguments: Vec<String>,
}

impl Arguments {
  pub(crate) fn try_parse_from_args(
    args: impl IntoIterator<Item = impl Into<OsString> + Clone>,
  ) -> Result<Self, clap::Error> {
    Parser::try_parse_from(args)
  }

  pub(crate) fn command() -> clap::Command {
    <Self as clap::CommandFactory>::command()
  }

  pub(crate) fn styles() -> Styles {
    Styles::styled()
      .error(AnsiColor::Red.on_default() | Effects::BOLD)
      .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
      .invalid(AnsiColor::Red.on_default())
      .literal(AnsiColor::Green.on_default())
      .placeholder(AnsiColor::Cyan.on_default())
      .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
      .valid(AnsiColor::Green.on_default())
  }

  fn parse_modulepath(values: &[String]) -> ConfigResult<Modulepath> {
    let path = if values.len() == 1 && values[0].contains(' ') {
      values[0].split_whitespace().collect::<Vec<&str>>()
    } else {
      values.iter().map(String::as_str).collect::<Vec<&str>>()
    };

    path
      .as_slice()
      .try_into()
      .map_err(|()| ConfigError::ModulePath {
        path: values.to_vec(),
      })
  }

  fn search_config(&self, positional: &Positional) -> ConfigResult<SearchConfig> {
    if self.global_justfile {
      return Ok(SearchConfig::GlobalJustfile);
    }

    let justfile = self.justfile.clone();
    let working_directory = self.working_directory.clone();

    if let Some(search_directory) = positional.search_directory.as_ref().map(PathBuf::from) {
      if justfile.is_some() || working_directory.is_some() {
        return Err(ConfigError::SearchDirConflict);
      }
      Ok(SearchConfig::FromSearchDirectory { search_directory })
    } else {
      match (justfile, working_directory) {
        (None, None) => Ok(SearchConfig::FromInvocationDirectory),
        (Some(justfile), None) => Ok(SearchConfig::WithJustfile { justfile }),
        (Some(justfile), Some(working_directory)) => {
          Ok(SearchConfig::WithJustfileAndWorkingDirectory {
            justfile,
            working_directory,
          })
        }
        (None, Some(_)) => Err(ConfigError::internal(
          "--working-directory set without --justfile",
        )),
      }
    }
  }

  pub(crate) fn into_config(self) -> ConfigResult<Config> {
    let mut overrides = BTreeMap::new();

    let mut set_iter = self.set.iter();
    while let Some(path) = set_iter.next() {
      overrides.insert(
        Config::parse_override(path)?,
        set_iter.next().unwrap().clone(),
      );
    }

    let positional = Positional::from_values(if self.arguments.is_empty() {
      None
    } else {
      Some(self.arguments.iter().map(String::as_str))
    });

    for (path, value) in &positional.overrides {
      overrides.insert(Config::parse_override(path)?, value.into());
    }

    let search_config = self.search_config(&positional)?;

    let format_overrides = || {
      overrides
        .iter()
        .map(|((path, key), value)| {
          if path.is_empty() {
            format!("{key}={value}")
          } else {
            format!("{path}::{key}={value}")
          }
        })
        .collect()
    };

    let argless: &[(&str, bool)] = &[
      ("CHANGELOG", self.changelog),
      ("DUMP", self.dump),
      ("EDIT", self.edit),
      ("FORMAT", self.format),
      ("JSON", self.json),
      ("INIT", self.init),
      ("MAN", self.man),
      ("SUMMARY", self.summary),
      ("VARIABLES", self.variables),
    ];

    for &(subcommand, active) in argless {
      if active {
        match (!overrides.is_empty(), !positional.arguments.is_empty()) {
          (false, false) => {}
          (true, false) => {
            return Err(ConfigError::SubcommandOverrides {
              subcommand,
              overrides: format_overrides(),
            });
          }
          (false, true) => {
            return Err(ConfigError::SubcommandArguments {
              arguments: positional.arguments,
              subcommand,
            });
          }
          (true, true) => {
            return Err(ConfigError::SubcommandOverridesAndArguments {
              arguments: positional.arguments,
              subcommand,
              overrides: format_overrides(),
            });
          }
        }
      }
    }

    let subcommand = if self.changelog {
      Subcommand::Changelog
    } else if self.choose {
      Subcommand::Choose {
        chooser: self.chooser.clone(),
      }
    } else if !self.command.is_empty() {
      let mut arguments = self.command;
      Subcommand::Command {
        binary: arguments.remove(0),
        arguments,
      }
    } else if let Some(shell) = self.completions {
      Subcommand::Completions { shell }
    } else if self.dump {
      Subcommand::Dump {
        format: self.dump_format,
      }
    } else if self.edit {
      Subcommand::Edit
    } else if self.evaluate {
      if positional.arguments.len() > 1 {
        return Err(ConfigError::SubcommandArguments {
          subcommand: "EVALUATE",
          arguments: positional
            .arguments
            .into_iter()
            .skip(1)
            .collect::<Vec<String>>(),
        });
      }

      Subcommand::Evaluate {
        variable: positional.arguments.into_iter().next(),
      }
    } else if self.format {
      Subcommand::Format
    } else if self.groups_cmd {
      Subcommand::Groups
    } else if self.init {
      Subcommand::Init
    } else if self.json {
      Subcommand::Dump {
        format: DumpFormat::Json,
      }
    } else if let Some(path) = self.list {
      Subcommand::List {
        path: if path.is_empty() {
          Modulepath::default()
        } else {
          Self::parse_modulepath(&path)?
        },
      }
    } else if self.man {
      Subcommand::Man
    } else if let Some(request) = self.request {
      Subcommand::Request {
        request: serde_json::from_str(&request)
          .map_err(|source| ConfigError::RequestParse { source })?,
      }
    } else if let Some(path) = self.show {
      Subcommand::Show {
        path: Self::parse_modulepath(&path)?,
      }
    } else if self.summary {
      Subcommand::Summary
    } else if let Some(path) = self.usage {
      Subcommand::Usage {
        path: Self::parse_modulepath(&path)?,
      }
    } else if self.variables {
      Subcommand::Variables
    } else {
      Subcommand::Run {
        arguments: positional.arguments,
      }
    };

    let unstable = self.unstable || subcommand == Subcommand::Summary;

    Ok(Config {
      alias_style: self.alias_style,
      allow_missing: self.allow_missing,
      ceiling: self.ceiling,
      check: self.check,
      color: self.color.into(),
      command_color: self.command_color.map(CommandColor::into),
      cygpath: self.cygpath,
      dotenv_filename: self.dotenv_filename,
      dotenv_path: self.dotenv_path,
      dry_run: self.dry_run,
      explain: self.explain,
      groups: self.group,
      highlight: !self.no_highlight,
      invocation_directory: env::current_dir().context(config_error::CurrentDirContext)?,
      list_heading: self.list_heading,
      list_prefix: self.list_prefix,
      list_submodules: self.list_submodules,
      load_dotenv: !self.no_dotenv,
      no_aliases: self.no_aliases,
      no_dependencies: self.no_deps,
      one: self.one,
      overrides,
      search_config,
      shell: self.shell,
      shell_args: if self.clear_shell_args {
        Some(Vec::new())
      } else if !self.shell_arg.is_empty() {
        Some(self.shell_arg)
      } else {
        None
      },
      shell_command: self.shell_command,
      subcommand,
      tempdir: self.tempdir,
      timestamp: self.timestamp,
      timestamp_format: self.timestamp_format,
      unsorted: self.unsorted,
      unstable,
      verbosity: if self.quiet {
        Verbosity::Quiet
      } else {
        Verbosity::from_flag_occurrences(self.verbose)
      },
      yes: self.yes,
    })
  }
}
