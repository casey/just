use {
  super::*,
  clap::{
    ArgAction, Args, Command, CommandFactory, Parser,
    builder::{
      FalseyValueParser, Styles,
      styling::{AnsiColor, Effects},
    },
  },
};

#[derive(Debug, Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(bin_name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(about = concat!(
  env!("CARGO_PKG_DESCRIPTION"),
  " - ",
  env!("CARGO_PKG_HOMEPAGE")
))]
#[command(trailing_var_arg = true)]
#[command(styles = styles())]
pub(crate) struct Arguments {
  #[arg(
    long,
    env = "JUST_ALIAS_STYLE",
    default_value = "right",
    value_enum,
    conflicts_with = "no_aliases",
    help = "Set list command alias display style"
  )]
  pub(crate) alias_style: AliasStyle,
  #[arg(
    long,
    env = "JUST_ALLOW_MISSING",
    help = "Ignore missing recipe and module errors"
  )]
  pub(crate) allow_missing: bool,
  #[arg(
    long,
    env = "JUST_CEILING",
    value_name = "CEILING",
    help = "Do not ascend above <CEILING> directory when searching for a justfile."
  )]
  pub(crate) ceiling: Option<PathBuf>,
  #[arg(
    long,
    requires = "fmt",
    help = "Run `--fmt` in 'check' mode. Exits with 0 if justfile is formatted correctly. Exits with 1 and prints a diff if formatting is required."
  )]
  pub(crate) check: bool,
  #[arg(
    long,
    env = "JUST_CHOOSER",
    help = "Override binary invoked by `--choose`"
  )]
  pub(crate) chooser: Option<String>,
  #[arg(long, overrides_with = "shell_arg", help = "Clear shell arguments")]
  pub(crate) clear_shell_args: bool,
  #[arg(
    long,
    env = "JUST_COLOR",
    default_value = "auto",
    value_enum,
    help = "Print colorful output"
  )]
  pub(crate) color: UseColor,
  #[arg(
    long,
    env = "JUST_COMMAND_COLOR",
    value_enum,
    help = "Echo recipe lines in <COMMAND-COLOR>"
  )]
  pub(crate) command_color: Option<CommandColor>,
  #[arg(
    long,
    env = "JUST_CYGPATH",
    default_value = "cygpath",
    value_name = "CYGPATH",
    help = "Use binary at <CYGPATH> to convert between unix and Windows paths."
  )]
  pub(crate) cygpath: PathBuf,
  #[arg(
    long,
    value_name = "DOTENV-FILENAME",
    conflicts_with = "dotenv_path",
    help = "Search for environment file named <DOTENV-FILENAME> instead of `.env`"
  )]
  pub(crate) dotenv_filename: Option<String>,
  #[arg(
    short = 'E',
    long,
    value_name = "DOTENV-PATH",
    help = "Load <DOTENV-PATH> as environment file instead of searching for one"
  )]
  pub(crate) dotenv_path: Option<PathBuf>,
  #[arg(
    short = 'n',
    long,
    env = "JUST_DRY_RUN",
    conflicts_with = "quiet",
    help = "Print what just would do without doing it"
  )]
  pub(crate) dry_run: bool,
  #[arg(
    long,
    env = "JUST_DUMP_FORMAT",
    default_value = "just",
    value_name = "FORMAT",
    value_enum,
    help = "Dump justfile as <FORMAT>"
  )]
  pub(crate) dump_format: DumpFormat,
  #[arg(
    long,
    env = "JUST_EXPLAIN",
    help = "Print recipe doc comment before running it"
  )]
  pub(crate) explain: bool,
  #[arg(
    long,
    short = 'g',
    conflicts_with = "justfile",
    conflicts_with = "working_directory",
    help = "Use global justfile"
  )]
  pub(crate) global_justfile: bool,
  #[arg(
    long,
    env = "JUST_HIGHLIGHT",
    overrides_with = "no_highlight",
    help = "Highlight echoed recipe lines in bold"
  )]
  pub(crate) highlight: bool,
  #[arg(
    short = 'f',
    long,
    env = "JUST_JUSTFILE",
    value_name = "JUSTFILE",
    help = "Use <JUSTFILE> as justfile"
  )]
  pub(crate) justfile: Option<PathBuf>,
  #[arg(
    long,
    env = "JUST_LIST_HEADING",
    default_value = "Available recipes:\n",
    value_name = "TEXT",
    help = "Print <TEXT> before list"
  )]
  pub(crate) list_heading: String,
  #[arg(
    long,
    env = "JUST_LIST_PREFIX",
    default_value = "    ",
    value_name = "TEXT",
    help = "Print <TEXT> before each list item"
  )]
  pub(crate) list_prefix: String,
  #[arg(
    long,
    env = "JUST_LIST_SUBMODULES",
    requires = "list",
    help = "List recipes in submodules"
  )]
  pub(crate) list_submodules: bool,
  #[arg(
    long = "group",
    env = "JUST_GROUP",
    requires = "list",
    help = "Only list recipes in <GROUP>"
  )]
  pub(crate) group: Vec<String>,
  #[arg(long, env = "JUST_NO_ALIASES", help = "Don't show aliases in list")]
  pub(crate) no_aliases: bool,
  #[arg(
    long = "no-deps",
    env = "JUST_NO_DEPS",
    alias = "no-dependencies",
    help = "Don't run recipe dependencies"
  )]
  pub(crate) no_deps: bool,
  #[arg(long, env = "JUST_NO_DOTENV", help = "Don't load `.env` file")]
  pub(crate) no_dotenv: bool,
  #[arg(
    long,
    env = "JUST_NO_HIGHLIGHT",
    overrides_with = "highlight",
    help = "Don't highlight echoed recipe lines in bold"
  )]
  pub(crate) no_highlight: bool,
  #[arg(
    long,
    env = "JUST_ONE",
    help = "Forbid multiple recipes from being invoked on the command line"
  )]
  pub(crate) one: bool,
  #[arg(
    short = 'q',
    long,
    env = "JUST_QUIET",
    conflicts_with = "dry_run",
    help = "Suppress all output"
  )]
  pub(crate) quiet: bool,
  #[arg(long, action = ArgAction::Append, num_args = 2, value_names = ["VARIABLE", "VALUE"], help = "Override <VARIABLE> with <VALUE>")]
  pub(crate) set: Vec<String>,
  #[arg(long, help = "Invoke <SHELL> to run recipes")]
  pub(crate) shell: Option<String>,
  #[arg(
    long,
    allow_hyphen_values = true,
    overrides_with = "clear_shell_args",
    help = "Invoke shell with <SHELL-ARG> as an argument"
  )]
  pub(crate) shell_arg: Vec<String>,
  #[arg(
    long,
    requires = "command",
    help = "Invoke <COMMAND> with the shell used to run recipe lines and backticks"
  )]
  pub(crate) shell_command: bool,
  #[arg(
    long,
    env = "JUST_TEMPDIR",
    value_name = "TEMPDIR",
    help = "Save temporary files to <TEMPDIR>."
  )]
  pub(crate) tempdir: Option<PathBuf>,
  #[arg(long, env = "JUST_TIMESTAMP", help = "Print recipe command timestamps")]
  pub(crate) timestamp: bool,
  #[arg(
    long,
    env = "JUST_TIMESTAMP_FORMAT",
    default_value = "%H:%M:%S",
    help = "Timestamp format string"
  )]
  pub(crate) timestamp_format: String,
  #[arg(
    short = 'u',
    long,
    env = "JUST_UNSORTED",
    help = "Return list and summary entries in source order"
  )]
  pub(crate) unsorted: bool,
  #[arg(long, env = "JUST_UNSTABLE", action = ArgAction::SetTrue, value_parser = FalseyValueParser::new(), help = "Enable unstable features")]
  pub(crate) unstable: bool,
  #[arg(short = 'v', long, env = "JUST_VERBOSE", action = ArgAction::Count, help = "Use verbose output")]
  pub(crate) verbose: u8,
  #[arg(
    short = 'd',
    long,
    env = "JUST_WORKING_DIRECTORY",
    value_name = "WORKING-DIRECTORY",
    requires = "justfile",
    help = "Use <WORKING-DIRECTORY> as working directory. --justfile must also be set"
  )]
  pub(crate) working_directory: Option<PathBuf>,
  #[arg(long, env = "JUST_YES", help = "Automatically confirm all recipes.")]
  pub(crate) yes: bool,
  #[command(flatten)]
  pub(crate) subcommand: SubcommandArguments,
  #[arg(action = ArgAction::Append, num_args = 1.., help = "Overrides and recipe(s) to run, defaulting to the first recipe in the justfile")]
  pub(crate) arguments: Vec<String>,
}

impl Arguments {
  pub(crate) fn app() -> Command {
    Self::command()
  }
}

#[derive(Args, Debug, Default)]
#[group(multiple = false)]
pub(crate) struct SubcommandArguments {
  #[arg(long, help_heading = cmd::HEADING, help = "Print changelog")]
  pub(crate) changelog: bool,
  #[arg(long, help_heading = cmd::HEADING, help = "Select one or more recipes to run using a binary chooser. If `--chooser` is not passed the chooser defaults to the value of $JUST_CHOOSER, falling back to `fzf`")]
  pub(crate) choose: bool,
  #[arg(short = 'c', long, action = ArgAction::Append, num_args = 1.., allow_hyphen_values = true, value_parser = clap::value_parser!(OsString), help_heading = cmd::HEADING, help = "Run an arbitrary command with the working directory, `.env`, overrides, and exports set")]
  pub(crate) command: Option<Vec<OsString>>,
  #[arg(long, value_name = "SHELL", value_enum, ignore_case = true, help_heading = cmd::HEADING, help = "Print shell completion script for <SHELL>")]
  pub(crate) completions: Option<completions::Shell>,
  #[arg(long, help_heading = cmd::HEADING, help = "Print justfile")]
  pub(crate) dump: bool,
  #[arg(short = 'e', long, help_heading = cmd::HEADING, help = "Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`")]
  pub(crate) edit: bool,
  #[arg(long, alias = "eval", help_heading = cmd::HEADING, help = "Evaluate and print all variables. If a variable name is given as an argument, only print that variable's value.")]
  pub(crate) evaluate: bool,
  #[arg(long = "fmt", alias = "format", help_heading = cmd::HEADING, help = "Format and overwrite justfile")]
  pub(crate) fmt: bool,
  #[arg(long = "groups", help_heading = cmd::HEADING, help = "List recipe groups")]
  pub(crate) groups: bool,
  #[arg(long, alias = "initialize", help_heading = cmd::HEADING, help = "Initialize new justfile in project root")]
  pub(crate) init: bool,
  #[arg(long, conflicts_with = "dump_format", help_heading = cmd::HEADING, help = "Print justfile as JSON")]
  pub(crate) json: bool,
  #[arg(short = 'l', long, num_args = 0.., value_name = "MODULE", conflicts_with = "arguments", help_heading = cmd::HEADING, help = "List available recipes in <MODULE> or root if omitted")]
  pub(crate) list: Option<Vec<String>>,
  #[arg(long, help_heading = cmd::HEADING, help = "Print man page")]
  pub(crate) man: bool,
  #[arg(long, hide = true, help_heading = cmd::REQUEST, help = "Execute <REQUEST>. For internal testing purposes only. May be changed or removed at any time.")]
  pub(crate) request: Option<String>,
  #[arg(short = 's', long, num_args = 1.., value_name = "PATH", conflicts_with = "arguments", help_heading = cmd::HEADING, help = "Show recipe at <PATH>")]
  pub(crate) show: Option<Vec<String>>,
  #[arg(long, help_heading = cmd::HEADING, help = "List names of available recipes")]
  pub(crate) summary: bool,
  #[arg(long, num_args = 1.., value_name = "PATH", conflicts_with = "arguments", help_heading = cmd::HEADING, help = "Print recipe usage information")]
  pub(crate) usage: Option<Vec<String>>,
  #[arg(long, help_heading = cmd::HEADING, help = "List names of variables")]
  pub(crate) variables: bool,
}

impl SubcommandArguments {
  pub(crate) fn argless(&self) -> Option<&'static str> {
    if self.changelog {
      Some(cmd::CHANGELOG)
    } else if self.dump {
      Some(cmd::DUMP)
    } else if self.edit {
      Some(cmd::EDIT)
    } else if self.fmt {
      Some(cmd::FORMAT)
    } else if self.init {
      Some(cmd::INIT)
    } else if self.json {
      Some(cmd::JSON)
    } else if self.man {
      Some(cmd::MAN)
    } else if self.summary {
      Some(cmd::SUMMARY)
    } else if self.variables {
      Some(cmd::VARIABLES)
    } else {
      None
    }
  }
}

fn styles() -> Styles {
  Styles::styled()
    .error(AnsiColor::Red.on_default() | Effects::BOLD)
    .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .invalid(AnsiColor::Red.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .valid(AnsiColor::Green.on_default())
}

pub(crate) mod cmd {
  pub(crate) const CHANGELOG: &str = "CHANGELOG";
  pub(crate) const DUMP: &str = "DUMP";
  pub(crate) const EDIT: &str = "EDIT";
  pub(crate) const EVALUATE: &str = "EVALUATE";
  pub(crate) const FORMAT: &str = "FORMAT";
  pub(crate) const INIT: &str = "INIT";
  pub(crate) const JSON: &str = "JSON";
  pub(crate) const MAN: &str = "MAN";
  pub(crate) const REQUEST: &str = "REQUEST";
  pub(crate) const SUMMARY: &str = "SUMMARY";
  pub(crate) const VARIABLES: &str = "VARIABLES";

  pub(crate) const HEADING: &str = "Commands";
}
