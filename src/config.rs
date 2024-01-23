use clap::{
  builder::{EnumValueParser, NonEmptyStringValueParser},
  parser::ValueSource,
  value_parser, ArgAction,
};
use clap_complete::Shell;

use {
  super::*,
  clap::{/*App, AppSettings,*/ Arg, ArgGroup, ArgMatches, /*ArgSettings*/ Command},
};

pub(crate) const CHOOSER_ENVIRONMENT_KEY: &str = "JUST_CHOOSER";

pub(crate) const CHOOSE_HELP: &str = "Select one or more recipes to run using a binary chooser. \
                                      If `--chooser` is not passed the chooser defaults to the \
                                      value of $JUST_CHOOSER, falling back to `fzf`";

pub(crate) fn chooser_default(justfile: &Path) -> OsString {
  let mut chooser = OsString::new();
  chooser.push("fzf --multi --preview 'just --unstable --color always --justfile \"");
  chooser.push(justfile);
  chooser.push("\" --show {}'");
  chooser
}

#[derive(Debug, PartialEq)]
pub(crate) struct Config {
  pub(crate) check: bool,
  pub(crate) color: Color,
  pub(crate) command_color: Option<ansi_term::Color>,
  pub(crate) dotenv_filename: Option<String>,
  pub(crate) dotenv_path: Option<PathBuf>,
  pub(crate) dry_run: bool,
  pub(crate) dump_format: DumpFormat,
  pub(crate) highlight: bool,
  pub(crate) invocation_directory: PathBuf,
  pub(crate) list_heading: String,
  pub(crate) list_prefix: String,
  pub(crate) load_dotenv: bool,
  pub(crate) no_dependencies: bool,
  pub(crate) search_config: SearchConfig,
  pub(crate) shell: Option<String>,
  pub(crate) shell_args: Option<Vec<String>>,
  pub(crate) shell_command: bool,
  pub(crate) subcommand: Subcommand,
  pub(crate) unsorted: bool,
  pub(crate) unstable: bool,
  pub(crate) verbosity: Verbosity,
  pub(crate) yes: bool,
}

mod cmd {
  pub(crate) const CHANGELOG: &str = "CHANGELOG";
  pub(crate) const CHOOSE: &str = "CHOOSE";
  pub(crate) const COMMAND: &str = "COMMAND";
  pub(crate) const COMPLETIONS: &str = "COMPLETIONS";
  pub(crate) const DUMP: &str = "DUMP";
  pub(crate) const EDIT: &str = "EDIT";
  pub(crate) const EVALUATE: &str = "EVALUATE";
  pub(crate) const FORMAT: &str = "FORMAT";
  pub(crate) const INIT: &str = "INIT";
  pub(crate) const LIST: &str = "LIST";
  pub(crate) const SHOW: &str = "SHOW";
  pub(crate) const SUMMARY: &str = "SUMMARY";
  pub(crate) const VARIABLES: &str = "VARIABLES";

  pub(crate) const ALL: &[&str] = &[
    CHANGELOG,
    CHOOSE,
    COMMAND,
    COMPLETIONS,
    DUMP,
    EDIT,
    EVALUATE,
    FORMAT,
    INIT,
    LIST,
    SHOW,
    SUMMARY,
    VARIABLES,
  ];

  pub(crate) const ARGLESS: &[&str] = &[
    CHANGELOG,
    COMPLETIONS,
    DUMP,
    EDIT,
    FORMAT,
    INIT,
    LIST,
    SHOW,
    SUMMARY,
    VARIABLES,
  ];
}

mod arg {
  pub(crate) const ARGUMENTS: &str = "ARGUMENTS";
  pub(crate) const CHECK: &str = "CHECK";
  pub(crate) const CHOOSER: &str = "CHOOSER";
  pub(crate) const CLEAR_SHELL_ARGS: &str = "CLEAR-SHELL-ARGS";
  pub(crate) const COLOR: &str = "COLOR";
  pub(crate) const COMMAND_COLOR: &str = "COMMAND-COLOR";
  pub(crate) const DOTENV_FILENAME: &str = "DOTENV-FILENAME";
  pub(crate) const DOTENV_PATH: &str = "DOTENV-PATH";
  pub(crate) const DRY_RUN: &str = "DRY-RUN";
  pub(crate) const DUMP_FORMAT: &str = "DUMP-FORMAT";
  pub(crate) const HIGHLIGHT: &str = "HIGHLIGHT";
  pub(crate) const JUSTFILE: &str = "JUSTFILE";
  pub(crate) const LIST_HEADING: &str = "LIST-HEADING";
  pub(crate) const LIST_PREFIX: &str = "LIST-PREFIX";
  pub(crate) const NO_DEPS: &str = "NO-DEPS";
  pub(crate) const NO_DOTENV: &str = "NO-DOTENV";
  pub(crate) const NO_HIGHLIGHT: &str = "NO-HIGHLIGHT";
  pub(crate) const QUIET: &str = "QUIET";
  pub(crate) const SET: &str = "SET";
  pub(crate) const SHELL: &str = "SHELL";
  pub(crate) const SHELL_ARG: &str = "SHELL-ARG";
  pub(crate) const SHELL_COMMAND: &str = "SHELL-COMMAND";
  pub(crate) const UNSORTED: &str = "UNSORTED";
  pub(crate) const UNSTABLE: &str = "UNSTABLE";
  pub(crate) const VERBOSE: &str = "VERBOSE";
  pub(crate) const WORKING_DIRECTORY: &str = "WORKING-DIRECTORY";
  pub(crate) const YES: &str = "YES";

  pub(crate) const COLOR_ALWAYS: &str = "always";
  pub(crate) const COLOR_AUTO: &str = "auto";
  pub(crate) const COLOR_NEVER: &str = "never";
  pub(crate) const COLOR_VALUES: [&str; 3] = [COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER];

  pub(crate) const COMMAND_COLOR_BLACK: &str = "black";
  pub(crate) const COMMAND_COLOR_BLUE: &str = "blue";
  pub(crate) const COMMAND_COLOR_CYAN: &str = "cyan";
  pub(crate) const COMMAND_COLOR_GREEN: &str = "green";
  pub(crate) const COMMAND_COLOR_PURPLE: &str = "purple";
  pub(crate) const COMMAND_COLOR_RED: &str = "red";
  pub(crate) const COMMAND_COLOR_YELLOW: &str = "yellow";
  pub(crate) const COMMAND_COLOR_VALUES: [&str; 7] = [
    COMMAND_COLOR_BLACK,
    COMMAND_COLOR_BLUE,
    COMMAND_COLOR_CYAN,
    COMMAND_COLOR_GREEN,
    COMMAND_COLOR_PURPLE,
    COMMAND_COLOR_RED,
    COMMAND_COLOR_YELLOW,
  ];

  pub(crate) const DUMP_FORMAT_JSON: &str = "json";
  pub(crate) const DUMP_FORMAT_JUST: &str = "just";
  pub(crate) const DUMP_FORMAT_VALUES: &[&str] = &[DUMP_FORMAT_JUST, DUMP_FORMAT_JSON];
}

impl Config {
  pub(crate) fn from_command_line_arguments(
    args: impl Iterator<Item = OsString>,
  ) -> ConfigResult<Self> {
    info!("Parsing command line arguments…");
    let app = Self::app();
    let matches = app.get_matches_from(args);
    Config::from_matches(matches)
  }

  pub(crate) fn generate_completions_script(shell: Shell) -> String {
    let buffer = Vec::new();
    let mut cursor = Cursor::new(buffer);

    let mut command = Self::app();
    clap_complete::generator::generate(shell, &mut command, env!("CARGO_PKG_NAME"), &mut cursor);

    let buffer = cursor.into_inner();
    String::from_utf8(buffer).expect("Clap completion not UTF-8")
  }

  fn app() -> Command {
    let app: Command = Command::new(env!("CARGO_PKG_NAME"))
      /*
      .setting(AppSettings::ColoredHelp)
      .setting(AppSettings::TrailingVarArg)
      */
      .arg(
        Arg::new(arg::CHECK)
          .long("check")
          .action(ArgAction::SetTrue)
          .requires(cmd::FORMAT)
          .help("Run `--fmt` in 'check' mode. Exits with 0 if justfile is formatted correctly. Exits with 1 and prints a diff if formatting is required."),
      )
      .arg(
        Arg::new(arg::CHOOSER)
          .long("chooser")
          .help("Override binary invoked by `--choose`"),
      )
      .arg(
        Arg::new(arg::COLOR)
          .long("color")
          .value_parser(arg::COLOR_VALUES)
          .default_value(arg::COLOR_AUTO)
          .help("Print colorful output"),
      )
      .arg(
        Arg::new(arg::COMMAND_COLOR)
          .long("command-color")
          .value_parser(arg::COMMAND_COLOR_VALUES)
          .help("Echo recipe lines in <COMMAND-COLOR>"),
      )
      .arg(Arg::new(arg::YES).long("yes").action(ArgAction::SetTrue).help("Automatically confirm all recipes."))
      .arg(
        Arg::new(arg::DRY_RUN)
          .short('n')
          .long("dry-run")
          .action(ArgAction::SetTrue)
          .help("Print what just would do without doing it")
          .conflicts_with(arg::QUIET),
      )
      .arg(
        Arg::new(arg::DUMP_FORMAT)
          .long("dump-format")
          .value_names(arg::DUMP_FORMAT_VALUES)
          .default_value(arg::DUMP_FORMAT_JUST)
          .value_name("FORMAT")
          .help("Dump justfile as <FORMAT>"),
      )
      .arg(
        Arg::new(arg::HIGHLIGHT)
          .long("highlight")
          .action(ArgAction::SetTrue)
          .help("Highlight echoed recipe lines in bold")
          .overrides_with(arg::NO_HIGHLIGHT),
      )
      .arg(
        Arg::new(arg::LIST_HEADING)
          .long("list-heading")
          .help("Print <TEXT> before list")
          .value_name("TEXT")
      )
      .arg(
        Arg::new(arg::LIST_PREFIX)
          .long("list-prefix")
          .help("Print <TEXT> before each list item")
          .value_name("TEXT")
      )
      .arg (
        Arg::new(arg::NO_DEPS)
          .long("no-deps")
          .alias("no-dependencies")
          .action(ArgAction::SetTrue)
          .help("Don't run recipe dependencies")
      )
      .arg(
        Arg::new(arg::NO_DOTENV)
          .long("no-dotenv")
          .action(ArgAction::SetTrue)
          .help("Don't load `.env` file"),
      )
      .arg(
        Arg::new(arg::NO_HIGHLIGHT)
          .long("no-highlight")
          .action(ArgAction::SetTrue)
          .help("Don't highlight echoed recipe lines in bold")
          .overrides_with(arg::HIGHLIGHT),
      )
      .arg(
        Arg::new(arg::JUSTFILE)
          .short('f')
          .long("justfile")
          .value_parser(value_parser!(PathBuf))
          .help("Use <JUSTFILE> as justfile"),
      )
      .arg(
        Arg::new(arg::QUIET)
          .short('q')
          .long("quiet")
          .action(ArgAction::SetTrue)
          .help("Suppress all output")
          .conflicts_with(arg::DRY_RUN),
      )
      .arg(
        Arg::new(arg::SET)
          .long("set")
          .num_args(2)
          .value_names(&["VARIABLE", "VALUE"])
          .action(ArgAction::Append)
          .help("Override <VARIABLE> with <VALUE>"),
      )
      .arg(
        Arg::new(arg::SHELL)
          .long("shell")
          .help("Invoke <SHELL> to run recipes"),
      )
      .arg(
        Arg::new(arg::SHELL_ARG)
          .long("shell-arg")
          .action(ArgAction::Append)
          .num_args(1)
          .allow_hyphen_values(true)
          .overrides_with(arg::CLEAR_SHELL_ARGS)
          .help("Invoke shell with <SHELL-ARG> as an argument"),
      )
      .arg(
        Arg::new(arg::SHELL_COMMAND)
          .long("shell-command")
          .action(ArgAction::SetTrue)
          .requires(cmd::COMMAND)
          .help("Invoke <COMMAND> with the shell used to run recipe lines and backticks"),
      )
      .arg(
        Arg::new(arg::CLEAR_SHELL_ARGS)
          .long("clear-shell-args")
          .action(ArgAction::SetTrue)
          .overrides_with(arg::SHELL_ARG)
          .help("Clear shell arguments"),
      )
      .arg(
        Arg::new(arg::UNSORTED)
          .long("unsorted")
          .short('u')
          .action(ArgAction::SetTrue)
          .help("Return list and summary entries in source order"),
      )
      .arg(
        Arg::new(arg::UNSTABLE)
          .long("unstable")
          .action(ArgAction::SetTrue)
          .help("Enable unstable features"),
      )
      .arg(
        Arg::new(arg::VERBOSE)
          .short('v')
          .long("verbose")
          .action(ArgAction::Count)
          .help("Use verbose output"),
      )
      .arg(
        Arg::new(arg::WORKING_DIRECTORY)
          .short('d')
          .long("working-directory")
          .value_parser(value_parser!(PathBuf))
          .help("Use <WORKING-DIRECTORY> as working directory. --justfile must also be set")
          .requires(arg::JUSTFILE),
      )
      .arg(
        Arg::new(cmd::CHANGELOG)
          .long("changelog")
          .action(ArgAction::SetTrue)
          .help("Print changelog"),
      )
      .arg(Arg::new(cmd::CHOOSE).long("choose").action(ArgAction::SetTrue).help(CHOOSE_HELP))
      .arg(
        Arg::new(cmd::COMMAND)
          .long("command")
          .short('c')
          .num_args(1)
          .allow_hyphen_values(true)
          .value_parser(value_parser!(OsString))
          .help(
            "Run an arbitrary command with the working directory, `.env`, overrides, and exports \
             set",
          ),
      )
      .arg(
        Arg::new(cmd::COMPLETIONS)
          .long("completions")
          .value_name("SHELL")
          .value_parser(EnumValueParser::<Shell>::new())
          .ignore_case(true)
          .help("Print shell completion script for <SHELL>"),
      )
      .arg(
        Arg::new(cmd::DUMP)
          .long("dump")
          .action(ArgAction::SetTrue)
          .help("Print justfile"),
      )
      .arg(
        Arg::new(cmd::EDIT)
          .short('e')
          .long("edit")
          .action(ArgAction::SetTrue)
          .help("Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`"),
      )
      .arg(Arg::new(cmd::EVALUATE).long("evaluate")
           .action(ArgAction::SetTrue)
           .help(
        "Evaluate and print all variables. If a variable name is given as an argument, only print \
         that variable's value.",
      ))
      .arg(
        Arg::new(cmd::FORMAT)
          .long("fmt")
          .alias("format")
          .action(ArgAction::SetTrue)
          .help("Format and overwrite justfile"),
      )
      .arg(
        Arg::new(cmd::INIT)
          .long("init")
          .alias("initialize")
          .action(ArgAction::SetTrue)
          .help("Initialize new justfile in project root"),
      )
      .arg(
        Arg::new(cmd::LIST)
          .short('l')
          .long("list")
          .action(ArgAction::SetTrue)
          .help("List available recipes and their arguments"),
      )
      .arg(
        Arg::new(cmd::SHOW)
          .short('s')
          .long("show")
          .value_name("RECIPE")
          .help("Show information about <RECIPE>"),
      )
      .arg(
        Arg::new(cmd::SUMMARY)
          .long("summary")
          .action(ArgAction::SetTrue)
          .help("List names of available recipes"),
      )
      .arg(
        Arg::new(cmd::VARIABLES)
          .long("variables")
          .action(ArgAction::SetTrue)
          .help("List names of variables"),
      )
      .arg(
        Arg::new(arg::DOTENV_FILENAME)
          .long("dotenv-filename")
          .help("Search for environment file named <DOTENV-FILENAME> instead of `.env`")
          .conflicts_with(arg::DOTENV_PATH),
      )
      .arg(
        Arg::new(arg::DOTENV_PATH)
          .long("dotenv-path")
          .value_parser(value_parser!(PathBuf))
          .help("Load environment file at <DOTENV-PATH> instead of searching for one")
      )
      .group(ArgGroup::new("SUBCOMMAND").args(cmd::ALL))
      .arg(
        Arg::new(arg::ARGUMENTS)
          .action(ArgAction::Append)
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
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(concat!(
          env!("CARGO_PKG_DESCRIPTION"),
          " - ",
          env!("CARGO_PKG_HOMEPAGE")
        ))
    }
  }

  fn color_from_matches(matches: &mut ArgMatches) -> ConfigResult<Color> {
    let value: String = matches
      .remove_one(arg::COLOR)
      .ok_or_else(|| ConfigError::Internal {
        message: "`--color` had no value".to_string(),
      })?;

    match value.as_ref() {
      arg::COLOR_AUTO => Ok(Color::auto()),
      arg::COLOR_ALWAYS => Ok(Color::always()),
      arg::COLOR_NEVER => Ok(Color::never()),
      _ => Err(ConfigError::Internal {
        message: format!("Invalid argument `{value}` to --color."),
      }),
    }
  }

  fn command_color_from_matches(
    matches: &mut ArgMatches,
  ) -> ConfigResult<Option<ansi_term::Color>> {
    if let Some(value) = matches.remove_one(arg::COMMAND_COLOR) {
      match value {
        arg::COMMAND_COLOR_BLACK => Ok(Some(ansi_term::Color::Black)),
        arg::COMMAND_COLOR_BLUE => Ok(Some(ansi_term::Color::Blue)),
        arg::COMMAND_COLOR_CYAN => Ok(Some(ansi_term::Color::Cyan)),
        arg::COMMAND_COLOR_GREEN => Ok(Some(ansi_term::Color::Green)),
        arg::COMMAND_COLOR_PURPLE => Ok(Some(ansi_term::Color::Purple)),
        arg::COMMAND_COLOR_RED => Ok(Some(ansi_term::Color::Red)),
        arg::COMMAND_COLOR_YELLOW => Ok(Some(ansi_term::Color::Yellow)),
        value => Err(ConfigError::Internal {
          message: format!("Invalid argument `{value}` to --command-color."),
        }),
      }
    } else {
      Ok(None)
    }
  }

  fn dump_format_from_matches(matches: &mut ArgMatches) -> ConfigResult<DumpFormat> {
    let value: String =
      matches
        .remove_one(arg::DUMP_FORMAT)
        .ok_or_else(|| ConfigError::Internal {
          message: "`--dump-format` had no value".to_string(),
        })?;

    match value.as_ref() {
      arg::DUMP_FORMAT_JSON => Ok(DumpFormat::Json),
      arg::DUMP_FORMAT_JUST => Ok(DumpFormat::Just),
      _ => Err(ConfigError::Internal {
        message: format!("Invalid argument `{value}` to --dump-format."),
      }),
    }
  }

  fn from_matches(mut matches: ArgMatches) -> ConfigResult<Self> {
    let invocation_directory = env::current_dir().context(config_error::CurrentDirContext)?;

    let verbosity = if matches.get_flag(arg::QUIET) {
      Verbosity::Quiet
    } else {
      Verbosity::from_flag_occurrences(matches.get_count(arg::VERBOSE))
    };

    let color = Self::color_from_matches(&mut matches)?;
    let command_color = Self::command_color_from_matches(&mut matches)?;

    let set_items = matches.get_occurrences::<String>(arg::SET);
    let mut overrides: BTreeMap<String, String> = BTreeMap::new();
    if let Some(items) = set_items {
      for mut values in items {
        overrides.insert(
          values.next().unwrap().to_owned(),
          values.next().unwrap().to_owned(),
        );
      }
    }

    let maybe_argument_occurrences: Option<Vec<String>> = matches
      .remove_occurrences::<String>(arg::ARGUMENTS)
      .map(|inner| inner.map(Iterator::collect).collect());
    let positional = Positional::from_values(maybe_argument_occurrences);

    for (name, value) in positional.overrides {
      overrides.insert(name.clone(), value.clone());
    }

    let search_config = {
      let justfile = matches.remove_one(arg::JUSTFILE);
      let working_directory = matches.remove_one(arg::WORKING_DIRECTORY);

      if let Some(search_directory) = positional.search_directory.map(PathBuf::from) {
        if justfile.is_some() || working_directory.is_some() {
          return Err(ConfigError::SearchDirConflict);
        }
        SearchConfig::FromSearchDirectory { search_directory }
      } else {
        match (justfile, working_directory) {
          (None, None) => SearchConfig::FromInvocationDirectory,
          (Some(justfile), None) => SearchConfig::WithJustfile { justfile },
          (Some(justfile), Some(working_directory)) => {
            SearchConfig::WithJustfileAndWorkingDirectory {
              justfile,
              working_directory,
            }
          }
          (None, Some(_)) => {
            return Err(ConfigError::internal(
              "--working-directory set without --justfile",
            ))
          }
        }
      }
    };

    for subcommand in cmd::ARGLESS {
      let present = matches!(
        matches.value_source(subcommand),
        Some(ValueSource::CommandLine)
      );
      if present {
        match (!overrides.is_empty(), !positional.arguments.is_empty()) {
          (false, false) => {}
          (true, false) => {
            return Err(ConfigError::SubcommandOverrides {
              subcommand,
              overrides,
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
              overrides,
            });
          }
        }
      }
    }

    let subcommand = if matches.get_flag(cmd::CHANGELOG) {
      Subcommand::Changelog
    } else if matches.get_flag(cmd::CHOOSE) {
      let chooser = matches.remove_one(arg::CHOOSER);
      Subcommand::Choose { chooser, overrides }
    } else if let Some(values) = matches.remove_occurrences::<OsString>(cmd::COMMAND) {
      let mut arguments = values.map(Iterator::collect).collect::<Vec<OsString>>();
      Subcommand::Command {
        binary: arguments.remove(0),
        arguments,
        overrides,
      }
    } else if let Some(shell) = matches.remove_one(cmd::COMPLETIONS) {
      Subcommand::Completions { shell }
    } else if matches.get_flag(cmd::EDIT) {
      Subcommand::Edit
    } else if matches.get_flag(cmd::SUMMARY) {
      Subcommand::Summary
    } else if matches.get_flag(cmd::DUMP) {
      Subcommand::Dump
    } else if matches.get_flag(cmd::FORMAT) {
      Subcommand::Format
    } else if matches.get_flag(cmd::INIT) {
      Subcommand::Init
    } else if matches.get_flag(cmd::LIST) {
      Subcommand::List
    } else if let Some(name) = matches.remove_one(cmd::SHOW) {
      Subcommand::Show { name }
    } else if matches.get_flag(cmd::EVALUATE) {
      if positional.arguments.len() > 1 {
        return Err(ConfigError::SubcommandArguments {
          subcommand: cmd::EVALUATE,
          arguments: positional
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
    } else if matches.get_flag(cmd::VARIABLES) {
      Subcommand::Variables
    } else {
      Subcommand::Run {
        arguments: positional.arguments,
        overrides,
      }
    };

    let shell_args = if matches.get_occurrences::<String>(arg::SHELL_ARG).is_some()
      || matches.get_flag(arg::CLEAR_SHELL_ARGS)
    {
      Some(
        matches
          .remove_occurrences::<String>(arg::SHELL_ARG)
          .map_or(Vec::new(), |shell_args| {
            shell_args.map(Iterator::collect).collect::<Vec<String>>()
          }),
      )
    } else {
      None
    };

    let unstable = matches.get_flag(arg::UNSTABLE)
      || env::var_os("JUST_UNSTABLE")
        .map(|val| !(val == "false" || val == "0" || val.is_empty()))
        .unwrap_or_default();

    Ok(Self {
      check: matches.get_flag(arg::CHECK),
      color,
      command_color,
      dotenv_filename: matches.remove_one(arg::DOTENV_FILENAME), //.map(str::to_owned),
      dotenv_path: matches.remove_one(arg::DOTENV_PATH),
      dry_run: matches.get_flag(arg::DRY_RUN),
      dump_format: Self::dump_format_from_matches(&mut matches)?,
      highlight: !matches.get_flag(arg::NO_HIGHLIGHT),
      invocation_directory,
      list_heading: matches
        .remove_one::<String>(arg::LIST_HEADING)
        .unwrap_or("Available recipes:\n".to_string())
        .to_owned(),
      list_prefix: matches
        .remove_one(arg::LIST_PREFIX)
        .unwrap_or("    ".to_string())
        .to_owned(),
      load_dotenv: !matches.get_flag(arg::NO_DOTENV),
      no_dependencies: matches.get_flag(arg::NO_DEPS),
      search_config,
      shell: matches.remove_one(arg::SHELL),
      shell_args,
      shell_command: matches.get_flag(arg::SHELL_COMMAND),
      subcommand,
      unsorted: matches.get_flag(arg::UNSORTED),
      unstable,
      verbosity,
      yes: matches.get_flag(arg::YES),
    })
  }

  pub(crate) fn require_unstable(&self, message: &str) -> Result<(), Error<'static>> {
    if self.unstable {
      Ok(())
    } else {
      Err(Error::Unstable {
        message: message.to_owned(),
      })
    }
  }

  pub(crate) fn run(self, loader: &Loader) -> Result<(), Error> {
    if let Err(error) = InterruptHandler::install(self.verbosity) {
      warn!("Failed to set CTRL-C handler: {error}");
    }

    self.subcommand.execute(&self, loader)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use pretty_assertions::assert_eq;

  macro_rules! test {
    {
      name: $name:ident,
      args: [$($arg:expr),*],
      $(color: $color:expr,)?
      $(dry_run: $dry_run:expr,)?
      $(dump_format: $dump_format:expr,)?
      $(highlight: $highlight:expr,)?
      $(no_dependencies: $no_dependencies:expr,)?
      $(search_config: $search_config:expr,)?
      $(shell: $shell:expr,)?
      $(shell_args: $shell_args:expr,)?
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
          $(dump_format: $dump_format,)?
          $(highlight: $highlight,)?
          $(no_dependencies: $no_dependencies,)?
          $(search_config: $search_config,)?
          $(shell: $shell,)?
          $(shell_args: $shell_args,)?
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
      .try_get_matches_from(arguments)
      .expect("argument parsing failed");
    let have = Config::from_matches(matches).expect("config parsing failed");
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

        app.try_get_matches_from(arguments).expect_err("Expected clap error");
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

        let matches = app.try_get_matches_from(arguments).expect("Matching fails");

        match Config::from_matches(matches).expect_err("config parsing succeeded") {
          $error => { $($check)? }
          other => panic!("Unexpected config error: {other}"),
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
    name: dry_run_long,
    args: ["--dry-run"],
    dry_run: true,
  }

  test! {
    name: dry_run_short,
    args: ["-n"],
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
    name: no_deps,
    args: ["--no-deps"],
    no_dependencies: true,
  }

  test! {
    name: no_dependencies,
    args: ["--no-dependencies"],
    no_dependencies: true,
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

  error! {
    name: dotenv_both_filename_and_path,
    args: ["--dotenv-filename", "foo", "--dotenv-path", "bar"],
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
    shell: None,
    shell_args: None,
  }

  test! {
    name: shell_set,
    args: ["--shell", "tclsh"],
    shell: Some("tclsh".to_owned()),
  }

  test! {
    name: shell_args_set,
    args: ["--shell-arg", "hello"],
    shell: None,
    shell_args: Some(vec!["hello".into()]),
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
    name: subcommand_conflict_changelog,
    args: ["--list", "--changelog"],
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
    name: subcommand_conflict_fmt,
    args: ["--list", "--fmt"],
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
    subcommand: Subcommand::Completions{shell: Shell::Bash },
  }

  test! {
    name: subcommand_completions_uppercase,
    args: ["--completions", "BASH"],
    subcommand: Subcommand::Completions{shell: Shell::Bash},
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
    name: dump_format,
    args: ["--dump-format", "json"],
    dump_format: DumpFormat::Json,
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
  }

  test! {
    name: shell_args_set_hyphen,
    args: ["--shell-arg", "--foo"],
    shell_args: Some(vec!["--foo".to_owned()]),
  }

  test! {
    name: shell_args_set_word,
    args: ["--shell-arg", "foo"],
    shell_args: Some(vec!["foo".to_owned()]),
  }

  test! {
    name: shell_args_set_multiple,
    args: ["--shell-arg", "foo", "--shell-arg", "bar"],
    shell_args: Some(vec!["foo".to_owned(), "bar".to_owned()]),

  }

  test! {
    name: shell_args_clear,
    args: ["--clear-shell-args"],
    shell_args: Some(vec![]),

  }

  test! {
    name: shell_args_clear_and_set,
    args: ["--clear-shell-args", "--shell-arg", "bar"],
    shell_args: Some(vec!["bar".to_owned()]),

  }

  test! {
    name: shell_args_set_and_clear,
    args: ["--shell-arg", "bar", "--clear-shell-args"],
    shell_args: Some(vec![]),

  }

  test! {
    name: shell_args_set_multiple_and_clear,
    args: ["--shell-arg", "bar", "--shell-arg", "baz", "--clear-shell-args"],
    shell_args: Some(vec![]),

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
    name: changelog_arguments,
    args: ["--changelog", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::CHANGELOG);
      assert_eq!(arguments, &["bar"]);
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
    name: fmt_arguments,
    args: ["--fmt", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::FORMAT);
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: fmt_alias,
    args: ["--format", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, cmd::FORMAT);
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
    name: init_alias,
    args: ["--initialize", "bar"],
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
}
