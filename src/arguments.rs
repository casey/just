use {
  super::*,
  clap::{
    ArgAction, Args, Parser,
    builder::{
      FalseyValueParser, Styles,
      styling::{AnsiColor, Effects},
    },
  },
};

#[derive(Debug, Parser)]
#[command(
  about = concat!(env!("CARGO_PKG_DESCRIPTION"), " - ", env!("CARGO_PKG_HOMEPAGE")),
  author = env!("CARGO_PKG_AUTHORS"),
  bin_name = env!("CARGO_PKG_NAME"),
  name = env!("CARGO_PKG_NAME"),
  styles = Styles::styled()
    .error(AnsiColor::Red.on_default() | Effects::BOLD)
    .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .invalid(AnsiColor::Red.on_default())
    .literal(AnsiColor::Green.on_default())
    .placeholder(AnsiColor::Cyan.on_default())
    .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
    .valid(AnsiColor::Green.on_default()),
  trailing_var_arg = true,
  version = env!("CARGO_PKG_VERSION"),
)]
pub struct Arguments {
  #[arg(
    conflicts_with = "no_aliases",
    default_value = "right",
    env = "JUST_ALIAS_STYLE",
    help = "Set list command alias display style",
    long,
    value_enum
  )]
  pub(crate) alias_style: AliasStyle,
  #[arg(
    env = "JUST_ALLOW_MISSING",
    help = "Ignore missing recipe and module errors",
    long
  )]
  pub(crate) allow_missing: bool,
  #[arg(
    add = ArgValueCompleter::new(Completer::complete_argument),
    help = "Overrides and recipe(s) to run, defaulting to the first recipe in the justfile",
    num_args = 1..,
  )]
  pub(crate) arguments: Vec<String>,
  #[arg(
    add = ArgValueCompleter::new(PathCompleter::dir()),
    env = "JUST_CEILING",
    help = "Do not ascend above <CEILING> directory when searching for a justfile.",
    long,
    value_name = "CEILING"
  )]
  pub(crate) ceiling: Option<PathBuf>,
  #[arg(
    help = "Run `--fmt` in 'check' mode. Exits with 0 if justfile is formatted correctly. \
            Exits with 1 and prints a diff if formatting is required.",
    long,
    requires = "fmt"
  )]
  pub(crate) check: bool,
  #[arg(
    add = ArgValueCompleter::new(PathCompleter::file()),
    env = "JUST_CHOOSER",
    help = "Override binary invoked by `--choose`",
    long
  )]
  pub(crate) chooser: Option<PathBuf>,
  #[arg(help = "Clear shell arguments", long, overrides_with = "shell_arg")]
  pub(crate) clear_shell_args: bool,
  #[arg(
    default_value = "auto",
    env = "JUST_COLOR",
    help = "Print colorful output",
    long,
    value_enum
  )]
  pub(crate) color: UseColor,
  #[arg(
    env = "JUST_COMMAND_COLOR",
    help = "Echo recipe lines in <COMMAND-COLOR>",
    long,
    value_enum
  )]
  pub(crate) command_color: Option<CommandColor>,
  #[arg(
    env = "JUST_COMPLETE_ALIASES",
    help = "Auto-complete recipe aliases",
    long
  )]
  pub(crate) complete_aliases: bool,
  #[arg(
    add = ArgValueCompleter::new(PathCompleter::file()),
    default_value = Self::DEFAULT_CYGPATH,
    env = "JUST_CYGPATH",
    help = "Use binary at <CYGPATH> to convert between unix and Windows paths.",
    long,
    value_name = "CYGPATH"
  )]
  pub(crate) cygpath: PathBuf,
  #[arg(
    conflicts_with = "dotenv_path",
    help = "Search for environment file named <DOTENV-FILENAME> instead of `.env`",
    long,
    value_name = "DOTENV-FILENAME"
  )]
  pub(crate) dotenv_filename: Option<String>,
  #[arg(
    add = ArgValueCompleter::new(PathCompleter::file()),
    help = "Load <DOTENV-PATH> as environment file instead of searching for one",
    long,
    short = 'E',
    value_name = "DOTENV-PATH"
  )]
  pub(crate) dotenv_path: Option<PathBuf>,
  #[arg(
    conflicts_with = "quiet",
    env = "JUST_DRY_RUN",
    help = "Print what just would do without doing it",
    long,
    short = 'n'
  )]
  pub(crate) dry_run: bool,
  #[arg(
    default_value = "just",
    env = "JUST_DUMP_FORMAT",
    help = "Dump justfile as <FORMAT>",
    long,
    value_enum,
    value_name = "FORMAT"
  )]
  pub(crate) dump_format: DumpFormat,
  #[arg(
    default_value = "just",
    env = "JUST_EVALUATE_FORMAT",
    help = "Print evaluated variables in <FORMAT>",
    long,
    value_enum,
    value_name = "FORMAT"
  )]
  pub(crate) evaluate_format: EvaluateFormat,
  #[arg(
    env = "JUST_EXPLAIN",
    help = "Print recipe doc comment before running it",
    long
  )]
  pub(crate) explain: bool,
  #[arg(
    conflicts_with = "justfile",
    conflicts_with = "working_directory",
    help = "Use global justfile",
    long,
    short = 'g'
  )]
  pub(crate) global_justfile: bool,
  #[arg(
    add = ArgValueCompleter::new(Completer::complete_group),
    env = "JUST_GROUP",
    help = "Only list recipes in <GROUP>",
    long = "group",
    requires = "list"
  )]
  pub(crate) group: Vec<String>,
  #[arg(
    env = "JUST_HIGHLIGHT",
    help = "Highlight echoed recipe lines in bold",
    long,
    overrides_with = "no_highlight"
  )]
  pub(crate) highlight: bool,
  #[arg(
    default_value = "    ",
    env = "JUST_INDENTATION",
    help = "Indent recipes bodies with <INDENTATION>",
    long
  )]
  pub(crate) indentation: Indentation,
  #[arg(
    add = ArgValueCompleter::new(PathCompleter::file()),
    env = "JUST_JUSTFILE",
    help = "Use <JUSTFILE> as justfile",
    long,
    short = 'f',
    value_name = "JUSTFILE"
  )]
  pub(crate) justfile: Option<PathBuf>,
  #[arg(
    env = "JUST_JUSTFILE_NAME",
    help = "Search for justfile named <NAME>",
    long = "justfile-name",
    value_name = "NAME"
  )]
  pub(crate) justfile_names: Option<Vec<String>>,
  #[arg(
    default_value = Arguments::DEFAULT_LIST_HEADING,
    env = "JUST_LIST_HEADING",
    help = "Print <TEXT> before list",
    long,
    value_name = "TEXT"
  )]
  pub(crate) list_heading: String,
  #[arg(
    default_value = Arguments::DEFAULT_LIST_PREFIX,
    env = "JUST_LIST_PREFIX",
    help = "Print <TEXT> before each list item",
    long,
    value_name = "TEXT"
  )]
  pub(crate) list_prefix: String,
  #[arg(
    env = "JUST_LIST_SUBMODULES",
    help = "List recipes in submodules",
    long,
    requires = "list"
  )]
  pub(crate) list_submodules: bool,
  #[arg(env = "JUST_NO_ALIASES", help = "Don't show aliases in list", long)]
  pub(crate) no_aliases: bool,
  #[arg(
    alias = "no-dependencies",
    env = "JUST_NO_DEPS",
    help = "Don't run recipe dependencies",
    long = "no-deps"
  )]
  pub(crate) no_deps: bool,
  #[arg(env = "JUST_NO_DOTENV", help = "Don't load `.env` file", long)]
  pub(crate) no_dotenv: bool,
  #[arg(
    env = "JUST_NO_HIGHLIGHT",
    help = "Don't highlight echoed recipe lines in bold",
    long,
    overrides_with = "highlight"
  )]
  pub(crate) no_highlight: bool,
  #[arg(
    env = "JUST_ONE",
    help = "Forbid multiple recipes from being invoked on the command line",
    long
  )]
  pub(crate) one: bool,
  #[arg(
    conflicts_with = "dry_run",
    env = "JUST_QUIET",
    help = "Suppress all output",
    long,
    short = 'q'
  )]
  pub(crate) quiet: bool,
  #[arg(
    add = ArgValueCompleter::new(Completer::complete_variable),
    help = "Override <VARIABLE> with <VALUE>",
    long,
    num_args = 2,
    value_names = ["VARIABLE", "VALUE"],
  )]
  pub(crate) set: Vec<String>,
  #[arg(help = "Invoke <SHELL> to run recipes", long)]
  pub(crate) shell: Option<String>,
  #[arg(
    allow_hyphen_values = true,
    help = "Invoke shell with <SHELL-ARG> as an argument",
    long,
    overrides_with = "clear_shell_args"
  )]
  pub(crate) shell_arg: Vec<String>,
  #[arg(
    help = "Invoke <COMMAND> with the shell used to run recipe lines and backticks",
    long,
    requires = "command"
  )]
  pub(crate) shell_command: bool,
  #[command(flatten)]
  pub(crate) subcommand: Subcommand,
  #[arg(
    add = ArgValueCompleter::new(PathCompleter::dir()),
    env = "JUST_TEMPDIR",
    help = "Save temporary files to <TEMPDIR>.",
    long,
    value_name = "TEMPDIR"
  )]
  pub(crate) tempdir: Option<PathBuf>,
  #[arg(env = "JUST_TIME", help = "Print recipe execution time", long)]
  pub(crate) time: bool,
  #[arg(env = "JUST_TIMESTAMP", help = "Print recipe command timestamps", long)]
  pub(crate) timestamp: bool,
  #[arg(
    default_value = Self::DEFAULT_TIMESTAMP_FORMAT,
    env = "JUST_TIMESTAMP_FORMAT",
    help = "Timestamp format string",
    long
  )]
  pub(crate) timestamp_format: String,
  #[arg(
    env = "JUST_UNSORTED",
    help = "Return list and summary entries in source order",
    long,
    short = 'u'
  )]
  pub(crate) unsorted: bool,
  #[arg(
    env = "JUST_UNSTABLE",
    help = "Enable unstable features",
    long,
    value_parser = FalseyValueParser::new(),
  )]
  pub(crate) unstable: bool,
  #[arg(
    action = ArgAction::Count,
    env = "JUST_VERBOSE",
    help = "Use verbose output",
    long,
    short = 'v',
  )]
  pub(crate) verbose: u8,
  #[arg(
    add = ArgValueCompleter::new(PathCompleter::dir()),
    env = "JUST_WORKING_DIRECTORY",
    help = "Use <WORKING-DIRECTORY> as working directory. --justfile must also be set",
    long,
    requires = "justfile",
    short = 'd',
    value_name = "WORKING-DIRECTORY"
  )]
  pub(crate) working_directory: Option<PathBuf>,
  #[arg(env = "JUST_YES", help = "Automatically confirm all recipes.", long)]
  pub(crate) yes: bool,
}

#[derive(Args, Debug, Default)]
#[group(multiple = false)]
pub(crate) struct Subcommand {
  #[arg(
    help = "Print changelog",
    help_heading = Self::HEADING,
    long,
  )]
  pub(crate) changelog: bool,
  #[arg(
    help = "Select one or more recipes to run using a binary chooser. If `--chooser` is not passed \
            the chooser defaults to the value of $JUST_CHOOSER, falling back to `fzf`",
    help_heading = Self::HEADING,
    long,
  )]
  pub(crate) choose: bool,
  #[arg(
    allow_hyphen_values = true,
    help = "Run an arbitrary command with the working directory, `.env`, overrides, and exports \
            set",
    help_heading = Self::HEADING,
    long,
    num_args = 1..,
    short = 'c',
    value_parser = clap::value_parser!(OsString),
  )]
  pub(crate) command: Option<Vec<OsString>>,
  #[arg(
    help = "Print shell completion script for <SHELL>",
    help_heading = Self::HEADING,
    ignore_case = true,
    long,
    value_enum,
    value_name = "SHELL",
  )]
  pub(crate) completions: Option<Shell>,
  #[arg(
    help = "Print justfile",
    help_heading = Self::HEADING,
    long,
  )]
  pub(crate) dump: bool,
  #[arg(
    help = "Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`",
    help_heading = Self::HEADING,
    long,
    short = 'e',
  )]
  pub(crate) edit: bool,
  #[arg(
    alias = "eval",
    help = "Evaluate and print all variables. If a variable name is given as an argument, only \
            print that variable's value.",
    help_heading = Self::HEADING,
    long,
  )]
  pub(crate) evaluate: bool,
  #[arg(
    alias = "format",
    help = "Format and overwrite justfile",
    help_heading = Self::HEADING,
    long = "fmt",
  )]
  pub(crate) fmt: bool,
  #[arg(
    help = "List recipe groups",
    help_heading = Self::HEADING,
    long = "groups",
  )]
  pub(crate) groups: bool,
  #[arg(
    alias = "initialize",
    help = "Initialize new justfile in project root",
    help_heading = Self::HEADING,
    long,
  )]
  pub(crate) init: bool,
  #[arg(
    conflicts_with = "dump_format",
    help = "Print justfile as JSON",
    help_heading = Self::HEADING,
    long,
  )]
  pub(crate) json: bool,
  #[arg(
    conflicts_with = "arguments",
    help = "List available recipes in <MODULE> or root if omitted",
    help_heading = Self::HEADING,
    long,
    num_args = 0..,
    short = 'l',
    value_name = "MODULE",
  )]
  pub(crate) list: Option<Vec<String>>,
  #[arg(
    help = "Print man page",
    help_heading = Self::HEADING,
    long,
  )]
  pub(crate) man: bool,
  #[arg(
    help = "Execute <REQUEST>. For internal testing purposes only. May be changed or removed at \
            any time.",
    help_heading = Self::HEADING,
    hide = true,
    long,
  )]
  pub(crate) request: Option<String>,
  #[arg(
    add = ArgValueCompleter::new(Completer::complete_recipe),
    conflicts_with = "arguments",
    help = "Show recipe at <PATH>",
    help_heading = Self::HEADING,
    long,
    num_args = 1..,
    short = 's',
    value_name = "PATH",
  )]
  pub(crate) show: Option<Vec<String>>,
  #[arg(
    help = "List names of available recipes",
    help_heading = Self::HEADING,
    long,
  )]
  pub(crate) summary: bool,
  #[arg(
    add = ArgValueCompleter::new(Completer::complete_recipe),
    conflicts_with = "arguments",
    help = "Print recipe usage information",
    help_heading = Self::HEADING,
    long,
    num_args = 1..,
    value_name = "PATH",
  )]
  pub(crate) usage: Option<Vec<String>>,
  #[arg(
    long,
    help_heading = Self::HEADING,
    help = "List names of variables",
  )]
  pub(crate) variables: bool,
}

impl Arguments {
  pub(crate) const DEFAULT_CYGPATH: &str = "cygpath";
  pub(crate) const DEFAULT_LIST_HEADING: &str = "Available recipes:\n";
  pub(crate) const DEFAULT_LIST_PREFIX: &str = "    ";
  pub(crate) const DEFAULT_TIMESTAMP_FORMAT: &str = "%H:%M:%S";
}

impl Subcommand {
  pub(crate) const HEADING: &str = "Commands";
}
