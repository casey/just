use clap::{
  builder::{PossibleValue, TypedValueParser},
  Arg, ArgAction, Args,
};

use {
  super::*,
  clap::{ArgMatches, Command, Parser}, //clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, ArgSettings},
};

pub(crate) const CHOOSER_ENVIRONMENT_KEY: &str = "JUST_CHOOSER";

/*
pub(crate) const CHOOSE_HELP: &str = "Select one or more recipes to run using a binary chooser. \
                                      If `--chooser` is not passed the chooser defaults to the \
                                      value of $JUST_CHOOSER, falling back to `fzf`";

                                      */
pub(crate) fn chooser_default(justfile: &Path) -> OsString {
  let mut chooser = OsString::new();
  chooser.push("fzf --multi --preview 'just --unstable --color always --justfile \"");
  chooser.push(justfile);
  chooser.push("\" --show {}'");
  chooser
}

#[derive(Debug, PartialEq, Default)]
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
  pub(crate) const COLOR_VALUES: &[&str] = &[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER];

  pub(crate) const COMMAND_COLOR_BLACK: &str = "black";
  pub(crate) const COMMAND_COLOR_BLUE: &str = "blue";
  pub(crate) const COMMAND_COLOR_CYAN: &str = "cyan";
  pub(crate) const COMMAND_COLOR_GREEN: &str = "green";
  pub(crate) const COMMAND_COLOR_PURPLE: &str = "purple";
  pub(crate) const COMMAND_COLOR_RED: &str = "red";
  pub(crate) const COMMAND_COLOR_YELLOW: &str = "yellow";
  pub(crate) const COMMAND_COLOR_VALUES: &[&str] = &[
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

#[derive(Debug, Clone, Copy, clap::ValueEnum, serde::Serialize, Default)]
#[serde(rename_all = "kebab-case")]
enum ColorValue {
  #[default]
  Auto,
  Always,
  Never,
}

const HELP_TEMPLATE: &str = r#"
{name} {version}
{author}
{about}

{usage}
{all-args}

  "#;

#[derive(Default, Clone)]
struct AnsiColorParser;

impl TypedValueParser for AnsiColorParser {
  type Value = ansi_term::Color;

  fn parse_ref(
    &self,
    cmd: &Command,
    arg: Option<&Arg>,
    value: &std::ffi::OsStr,
  ) -> Result<Self::Value, clap::Error> {
    use clap::error::{ContextKind, ContextValue, ErrorKind};

    match value.to_string_lossy().as_ref() {
      arg::COMMAND_COLOR_BLACK => Ok(ansi_term::Color::Black),
      arg::COMMAND_COLOR_BLUE => Ok(ansi_term::Color::Blue),
      arg::COMMAND_COLOR_CYAN => Ok(ansi_term::Color::Cyan),
      arg::COMMAND_COLOR_GREEN => Ok(ansi_term::Color::Green),
      arg::COMMAND_COLOR_PURPLE => Ok(ansi_term::Color::Purple),
      arg::COMMAND_COLOR_RED => Ok(ansi_term::Color::Red),
      arg::COMMAND_COLOR_YELLOW => Ok(ansi_term::Color::Yellow),
      value => {
        let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
        if let Some(arg) = arg {
          err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(arg.to_string()),
          );
        }
        err.insert(
          ContextKind::InvalidValue,
          ContextValue::String(value.to_string()),
        );
        Err(err)
      }
    }
  }

  fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
    Some(Box::new(
      arg::COMMAND_COLOR_VALUES.iter().map(PossibleValue::new),
    ))
  }
}

#[derive(Default, Clone)]
struct ColorParser;
impl TypedValueParser for ColorParser {
  type Value = Color;

  fn parse_ref(
    &self,
    cmd: &Command,
    arg: Option<&Arg>,
    value: &std::ffi::OsStr,
  ) -> Result<Self::Value, clap::Error> {
    use clap::error::{ContextKind, ContextValue, ErrorKind};

    match value.to_string_lossy().as_ref() {
      arg::COLOR_AUTO => Ok(Color::auto()),
      arg::COLOR_ALWAYS => Ok(Color::always()),
      arg::COLOR_NEVER => Ok(Color::never()),
      value => {
        let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
        if let Some(arg) = arg {
          err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(arg.to_string()),
          );
        }
        err.insert(
          ContextKind::InvalidValue,
          ContextValue::String(value.to_string()),
        );
        Err(err)
      }
    }
  }

  fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
    Some(Box::new(arg::COLOR_VALUES.iter().map(PossibleValue::new)))
  }
}

#[derive(Default, Clone)]
struct DumpFormatParser;
impl TypedValueParser for DumpFormatParser {
  type Value = DumpFormat;
  fn parse_ref(
    &self,
    cmd: &Command,
    arg: Option<&Arg>,
    value: &std::ffi::OsStr,
  ) -> Result<Self::Value, clap::Error> {
    use clap::error::{ContextKind, ContextValue, ErrorKind};
    match value.to_string_lossy().as_ref() {
      arg::DUMP_FORMAT_JSON => Ok(DumpFormat::Json),
      arg::DUMP_FORMAT_JUST => Ok(DumpFormat::Just),
      value => {
        let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
        if let Some(arg) = arg {
          err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(arg.to_string()),
          );
        }
        err.insert(
          ContextKind::InvalidValue,
          ContextValue::String(value.to_string()),
        );
        Err(err)
      }
    }
  }

  fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
    Some(Box::new(
      arg::DUMP_FORMAT_VALUES.iter().map(PossibleValue::new),
    ))
  }
}

#[derive(Parser)]
#[command(version=clap::crate_version!(),
author = clap::crate_authors!(),
about = concat!(
            env!("CARGO_PKG_DESCRIPTION"),
            " - ",
            env!("CARGO_PKG_HOMEPAGE")),
help_template = HELP_TEMPLATE,
)]
struct CliArgs {
  /// Print changelog
  #[arg(long)]
  changelog: bool,

  ///Run `--fmt` in 'check' mode. Exits with 0 if justfile is formatted correctly. Exits with 1 and prints a diff if formatting is required.
  #[arg(long, requires = "fmt")]
  check: bool,

  ///Select one or more recipes to run using a binary chooser.
  /// If `--chooser` is not passed the chooser defaults to the \
  /// value of $JUST_CHOOSER, falling back to `fzf`";
  #[arg(long)]
  choose: bool,

  ///Override binary invoked by `--choose`
  #[arg(long)]
  chooser: Option<String>,

  ///Run an arbitrary command with the working directory, `.env`, overrides, and exports set
  #[arg(short='c', long, allow_hyphen_values=true, num_args=1..)]
  command: Option<Vec<OsString>>,

  ///Invoke shell with <SHELL-ARG> as an argument
  #[arg(
    long = "shell-args",
    allow_hyphen_values = true,
    num_args = 1,
    overrides_with = "clear_shell_args"
  )]
  shell_args: Vec<String>,

  ///Clear shell arguments
  #[arg(long = "clear-shell-args", overrides_with = "shell_args")]
  clear_shell_args: bool,

  ///Print colorful output
  #[arg(long, value_parser=ColorParser, default_value = "auto")]
  color: Color,

  ///Echo recipe lines in <COMMAND-COLOR>
  #[arg(long, value_parser=AnsiColorParser)]
  command_color: Option<ansi_term::Color>,

  ///Print shell completion script for <SHELL>
  #[arg(long, value_name = "SHELL")]
  completions: Option<String>,

  ///Search for environment file named <DOTENV-FILENAME> instead of `.env`
  #[arg(long = "dotenv-filename", conflicts_with = "dotenv_path")]
  dotenv_filename: Option<String>,

  ///Load environment file at <DOTENV-PATH> instead of searching for one
  #[arg(long = "dotenv-path")]
  dotenv_path: Option<PathBuf>,

  /// Print justfile
  #[arg(long)]
  dump: bool,

  ///Dump justfile as <FORMAT>
  #[arg(long="dump-format", default_value="just", value_parser=DumpFormatParser)]
  dump_format: DumpFormat,

  ///Print what just would do without doing it
  #[arg(short = 'n', long = "dry-run", conflicts_with = "quiet")]
  dry_run: bool,

  ///Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`
  #[arg(short = 'e', long)]
  edit: bool,

  ///Evaluate and print all variables. If a variable name is given as an argument, only print that variable's value.
  #[arg(long)]
  evaluate: bool,

  ///Format and overwrite justfile"
  #[arg(long, alias = "format")]
  fmt: bool,

  ///Highlight echoed recipe lines in bold
  #[arg(long, overrides_with = "no_highlight")]
  highlight: bool,

  ///Initialize new justfile in project root"
  #[arg(long, alias = "initialize")]
  init: bool,

  ///Use <JUSTFILE> as justfile
  #[arg(short = 'f', long)]
  justfile: Option<PathBuf>,

  ///List available recipes and their arguments
  #[arg(short, long)]
  list: bool,

  ///Don't run recipe dependencies
  #[arg(long="no-deps", alias="no-dependencies")]
  no_deps: bool,

  ///Don't load `.env` file"
  #[arg(long="no-dotenv")]
  no_dotenv: bool,

  ///Don't highlight echoed recipe lines in bold
  #[arg(long = "no-highlight")]
  no_highlight: bool,

  ///Suppress all output
  #[arg(short = 'q', long, conflicts_with = "dry_run")]
  quiet: bool,

  ///Show information about <RECIPE>
  #[arg(short = 's', long)]
  show: Option<String>,

  ///List names of available recipes
  #[arg(long)]
  summary: bool,

  ///Return list and summary entries in source order
  #[arg(short = 'u', long)]
  unsorted: bool,

  ///Enable unstable features
  #[arg(long)]
  unstable: bool,

  ///List names of variables
  #[arg(long)]
  variables: bool,

  ///Use verbose output
  #[arg(short='v', long, action=ArgAction::Count)]
  verbose: u8,

  ///Use <WORKING-DIRECTORY> as working directory. --justfile must also be set
  #[arg(short = 'd', long = "working-directory", requires = "justfile")]
  working_directory: Option<PathBuf>,

  ///Overrides and recipe(s) to run, defaulting to the first recipe in the justfile
  #[arg()]
  arguments: Vec<String>,

  ///Automatically confirm all recipes
  #[arg(long)]
  yes: bool,
}

impl Config {
  pub(crate) fn from_command_line_arguments(
    args: impl Iterator<Item = OsString>,
  ) -> ConfigResult<Self> {
    info!("Parsing command line arguments…");

    let command = Self::command();
    let command = CliArgs::augment_args(command);
    let matches = command.get_matches_from(args);
    Config::from_matches(matches)
    /*
    let app = Self::app();
    let matches = app.get_matches_from(args);
    Config::from_matches(&matches)
    */
  }

  pub(crate) fn generate_completions_script(shell: clap_complete::Shell) -> String {
      let command = Self::command();
      let mut command = CliArgs::augment_args(command);


    let buffer = Vec::new();
    let mut cursor = Cursor::new(buffer);

    clap_complete::generate(shell, &mut command,"just", &mut cursor);
    //Config::app().gen_completions_to(env!("CARGO_PKG_NAME"), shell, &mut cursor);

    let buffer = cursor.into_inner();
    String::from_utf8(buffer).expect("Clap completion not UTF-8")
  }

  fn command() -> clap::Command {
    Command::new(env!("CARGO_PKG_NAME"))
      .disable_colored_help(false)
      .arg(
        Arg::new("set")
          .long("set")
          .number_of_values(2)
          .action(ArgAction::Append)
          .value_names(&["VARIABLE", "VALUE"])
          .help("Override <VARIABLE> with <VALUE>"),
      )
  }

  /*
    fn app() -> App<'static, 'static> {
      let app = App::new(env!("CARGO_PKG_NAME"))
        .help_message("Print help information")
        .version_message("Print version information")
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::TrailingVarArg)
        .arg(
          Arg::with_name(arg::CHECK)
            .long("check")
            .requires(cmd::FORMAT)
            .help("Run `--fmt` in 'check' mode. Exits with 0 if justfile is formatted correctly. Exits with 1 and prints a diff if formatting is required."),
        )
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
          Arg::with_name(arg::COMMAND_COLOR)
            .long("command-color")
            .takes_value(true)
            .possible_values(arg::COMMAND_COLOR_VALUES)
            .help("Echo recipe lines in <COMMAND-COLOR>"),
        )
        .arg(Arg::with_name(arg::YES).long("yes").help("Automatically confirm all recipes."))
        .arg(
          Arg::with_name(arg::DRY_RUN)
            .short("n")
            .long("dry-run")
            .help("Print what just would do without doing it")
            .conflicts_with(arg::QUIET),
        )
        .arg(
          Arg::with_name(arg::DUMP_FORMAT)
            .long("dump-format")
            .takes_value(true)
            .possible_values(arg::DUMP_FORMAT_VALUES)
            .default_value(arg::DUMP_FORMAT_JUST)
            .value_name("FORMAT")
            .help("Dump justfile as <FORMAT>"),
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
        .arg (
          Arg::with_name(arg::NO_DEPS)
            .long("no-deps")
            .alias("no-dependencies")
            .help("Don't run recipe dependencies")
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
            .help("Invoke <SHELL> to run recipes"),
        )
        .arg(
          Arg::with_name(arg::SHELL_ARG)
            .long("shell-arg")
            .takes_value(true)
            .multiple(true)
            .number_of_values(1)
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
          Arg::with_name(arg::UNSTABLE)
            .long("unstable")
            .help("Enable unstable features"),
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
        .arg(
          Arg::with_name(cmd::CHANGELOG)
            .long("changelog")
            .help("Print changelog"),
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
            .help("Print justfile"),
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
          Arg::with_name(cmd::FORMAT)
            .long("fmt")
            .alias("format")
            .help("Format and overwrite justfile"),
        )
        .arg(
          Arg::with_name(cmd::INIT)
            .long("init")
            .alias("initialize")
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
        .arg(
          Arg::with_name(arg::DOTENV_FILENAME)
            .long("dotenv-filename")
            .takes_value(true)
            .help("Search for environment file named <DOTENV-FILENAME> instead of `.env`")
            .conflicts_with(arg::DOTENV_PATH),
        )
        .arg(
          Arg::with_name(arg::DOTENV_PATH)
            .long("dotenv-path")
            .help("Load environment file at <DOTENV-PATH> instead of searching for one")
            .takes_value(true),
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
          .version(env!("CARGO_PKG_VERSION"))
          .author(env!("CARGO_PKG_AUTHORS"))
          .about(concat!(
            env!("CARGO_PKG_DESCRIPTION"),
            " - ",
            env!("CARGO_PKG_HOMEPAGE")
          ))
      }
    }
  */

  fn from_matches(mut matches: ArgMatches) -> ConfigResult<Self> {
    let invocation_directory = env::current_dir().context(config_error::CurrentDirContext)?;

    let verbosity = if matches.contains_id("quiet") {
      Verbosity::Quiet
    } else {
      Verbosity::from_flag_occurrences(matches.get_count("verbose"))
    };

    let color: Color = matches.remove_one("color").unwrap();
    let command_color = matches.remove_one("command_color");
    let mut overrides = BTreeMap::new();

    if let Some(occurrences) = matches.get_occurrences::<String>("set") {
      for mut values in occurrences {
        overrides.insert(
          values.next().unwrap().to_string(),
          values.next().unwrap().to_string(),
        );
      }
    }

    let override_items: Option<Vec<_>> = matches
      .get_many("arguments")
      .map(|item| item.map(|s: &String| s.as_ref()).collect());
    let positional = Positional::from_values(override_items);

    for (name, value) in positional.overrides {
      overrides.insert(name.clone(), value.clone());
    }

    let search_config = {
      let justfile: Option<PathBuf> = matches.remove_one("justfile");
      let working_directory: Option<PathBuf> = matches.remove_one("working_directory");

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

    let argless = [
      "changelog",
      "completions",
      "dump",
      "edit",
      "format",
      "init",
      "list",
      "show",
      "summary",
      "variables",
    ];

    for subcommand in argless {
      if matches.contains_id(subcommand) {
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

    let shell_arg_count = matches
      .get_occurrences::<Vec<String>>("shell_args")
      .map(|occ| occ.count())
      .unwrap_or(0);

    let clear_shell_arg_count = matches
      .get_occurrences::<Vec<String>>("clear_shell_args")
      .map(|occ| occ.count())
      .unwrap_or(0);

    let shell_args = if shell_arg_count > 0 || clear_shell_arg_count > 0 {
      Some(
        matches
          .remove_many("shell_args")
          .map_or(Vec::new(), |shell_args| {
            shell_args.map(str::to_owned).collect()
          }),
      )
    } else {
      None
    };

    let subcommand = if matches.get_flag("changelog") {
      Subcommand::Changelog
    } else if let Some(chooser) = matches.remove_one("chooser") {
      Subcommand::Choose { chooser, overrides }
    } else if let Some(values) = matches.remove_many("command") {
      let mut arguments = values.map(OsStr::to_owned).collect::<Vec<OsString>>();
      Subcommand::Command {
        binary: arguments.remove(0),
        arguments,
        overrides,
      }
    } else if let Some(shell) = matches.remove_one("completions") {
      Subcommand::Completions { shell }
    } else if matches.get_flag("edit") {
      Subcommand::Edit
    } else if matches.get_flag("summary") {
      Subcommand::Summary
    } else if matches.get_flag("dump") {
      Subcommand::Dump
    } else if matches.get_flag("fmt") {
      Subcommand::Format
    } else if matches.get_flag("init") {
      Subcommand::Init
    } else if matches.get_flag("list") {
      Subcommand::List
    } else if let Some(name) = matches.remove_one("show") {
      Subcommand::Show { name }
    } else if matches.get_flag("evaluate") {
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
    } else if matches.get_flag("variables") {
      Subcommand::Variables
    } else {
      Subcommand::Run {
        arguments: positional.arguments,
        overrides,
      }
    };

    let unstable = matches.contains_id("unstable")
      || env::var_os("JUST_UNSTABLE")
        .map(|val| !(val == "false" || val == "0" || val.is_empty()))
        .unwrap_or_default();

    Ok(Self {
      check: matches.contains_id("check"),
      color,
      command_color,
      dotenv_filename: matches.remove_one("dotenv_filename"),
      dotenv_path: matches.remove_one("dotenv_path"),
      dry_run: matches.get_flag("dry_run"),
      dump_format: matches.remove_one("dump_format").unwrap(),
      highlight: !matches.get_flag("no_highlight"),
      invocation_directory,
      /*
      list_heading: matches
        .value_of(arg::LIST_HEADING)
        .unwrap_or("Available recipes:\n")
        .to_owned(),
      list_prefix: matches
        .value_of(arg::LIST_PREFIX)
        .unwrap_or("    ")
        .to_owned(),
        */
      load_dotenv: !matches.get_flag("no_dotenv"),
      no_dependencies: matches.get_flag("no_deps"),
      search_config,
      //shell: matches.value_of(arg::SHELL).map(str::to_owned),
      shell_args,
      /*
      shell_command: matches.is_present(arg::SHELL_COMMAND),
      */
      subcommand,
      unsorted: matches.get_flag("unsorted"),
      unstable,
      verbosity,
      yes: matches.get_flag("yes"),
      ..Default::default()
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
      .get_matches_from_safe(arguments)
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

        let matches = app.get_matches_from_safe(arguments).expect("Matching fails");

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
