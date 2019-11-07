use crate::common::*;

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches};
use unicode_width::UnicodeWidthStr;

pub(crate) const DEFAULT_SHELL: &str = "sh";

#[derive(Debug, PartialEq)]
pub(crate) struct Config {
  pub(crate) arguments: Vec<String>,
  pub(crate) color: Color,
  pub(crate) dry_run: bool,
  pub(crate) highlight: bool,
  pub(crate) invocation_directory: PathBuf,
  pub(crate) overrides: BTreeMap<String, String>,
  pub(crate) quiet: bool,
  pub(crate) search_config: SearchConfig,
  pub(crate) shell: String,
  pub(crate) subcommand: Subcommand,
  pub(crate) verbosity: Verbosity,
}

mod cmd {
  pub(crate) const DUMP: &str = "DUMP";
  pub(crate) const EDIT: &str = "EDIT";
  pub(crate) const EVALUATE: &str = "EVALUATE";
  pub(crate) const LIST: &str = "LIST";
  pub(crate) const SHOW: &str = "SHOW";
  pub(crate) const SUMMARY: &str = "SUMMARY";
}

mod arg {
  pub(crate) const ARGUMENTS: &str = "ARGUMENTS";
  pub(crate) const COLOR: &str = "COLOR";
  pub(crate) const DRY_RUN: &str = "DRY-RUN";
  pub(crate) const HIGHLIGHT: &str = "HIGHLIGHT";
  pub(crate) const NO_HIGHLIGHT: &str = "NO-HIGHLIGHT";
  pub(crate) const JUSTFILE: &str = "JUSTFILE";
  pub(crate) const QUIET: &str = "QUIET";
  pub(crate) const SET: &str = "SET";
  pub(crate) const SHELL: &str = "SHELL";
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
        Arg::with_name(arg::ARGUMENTS)
          .multiple(true)
          .help("The recipe(s) to run, defaults to the first recipe in the justfile"),
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
      .arg(
        Arg::with_name(cmd::EVALUATE)
          .long("evaluate")
          .help("Print evaluated variables"),
      )
      .arg(
        Arg::with_name(arg::HIGHLIGHT)
          .long("highlight")
          .help("Highlight echoed recipe lines in bold")
          .overrides_with(arg::NO_HIGHLIGHT),
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
          .help("Use <JUSTFILE> as justfile."),
      )
      .arg(
        Arg::with_name(cmd::LIST)
          .short("l")
          .long("list")
          .help("List available recipes and their arguments"),
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
          .help("Set <VARIABLE> to <VALUE>"),
      )
      .arg(
        Arg::with_name(arg::SHELL)
          .long("shell")
          .takes_value(true)
          .default_value(DEFAULT_SHELL)
          .help("Invoke <SHELL> to run recipes"),
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
      .group(ArgGroup::with_name("EARLY-EXIT").args(&[
        arg::ARGUMENTS,
        cmd::DUMP,
        cmd::EDIT,
        cmd::EVALUATE,
        cmd::LIST,
        cmd::SHOW,
        cmd::SUMMARY,
      ]));

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

  pub(crate) fn from_matches(matches: &ArgMatches) -> ConfigResult<Config> {
    let invocation_directory = env::current_dir().context(config_error::CurrentDir)?;

    let verbosity = Verbosity::from_flag_occurrences(matches.occurrences_of(arg::VERBOSE));

    let color = Self::color_from_value(
      matches
        .value_of(arg::COLOR)
        .expect("`--color` had no value"),
    )?;

    let subcommand = if matches.is_present(cmd::EDIT) {
      Subcommand::Edit
    } else if matches.is_present(cmd::SUMMARY) {
      Subcommand::Summary
    } else if matches.is_present(cmd::DUMP) {
      Subcommand::Dump
    } else if matches.is_present(cmd::LIST) {
      Subcommand::List
    } else if matches.is_present(cmd::EVALUATE) {
      Subcommand::Evaluate
    } else if let Some(name) = matches.value_of(cmd::SHOW) {
      Subcommand::Show {
        name: name.to_owned(),
      }
    } else {
      Subcommand::Run
    };

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

    fn is_override(arg: &&str) -> bool {
      arg.chars().skip(1).any(|c| c == '=')
    }

    let raw_arguments: Vec<&str> = matches
      .values_of(arg::ARGUMENTS)
      .map(Iterator::collect)
      .unwrap_or_default();

    for argument in raw_arguments.iter().cloned().take_while(is_override) {
      let i = argument
        .char_indices()
        .skip(1)
        .find(|&(_, c)| c == '=')
        .unwrap()
        .0;

      let name = argument[..i].to_owned();
      let value = argument[i + 1..].to_owned();

      overrides.insert(name, value);
    }

    let mut search_directory = None;

    let arguments = raw_arguments
      .into_iter()
      .skip_while(is_override)
      .enumerate()
      .flat_map(|(i, argument)| {
        if i == 0 {
          if let Some(i) = argument.rfind('/') {
            let (dir, recipe) = argument.split_at(i + 1);

            search_directory = Some(PathBuf::from(dir));

            if recipe.is_empty() {
              return None;
            } else {
              return Some(recipe);
            }
          }
        }

        Some(argument)
      })
      .map(|argument| argument.to_owned())
      .collect::<Vec<String>>();

    let search_config = {
      let justfile = matches.value_of(arg::JUSTFILE).map(PathBuf::from);
      let working_directory = matches.value_of(arg::WORKING_DIRECTORY).map(PathBuf::from);

      if let Some(search_directory) = search_directory {
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

    Ok(Config {
      dry_run: matches.is_present(arg::DRY_RUN),
      highlight: !matches.is_present(arg::NO_HIGHLIGHT),
      quiet: matches.is_present(arg::QUIET),
      shell: matches.value_of(arg::SHELL).unwrap().to_owned(),
      search_config,
      invocation_directory,
      subcommand,
      verbosity,
      color,
      overrides,
      arguments,
    })
  }

  pub(crate) fn run_subcommand(self) -> Result<(), i32> {
    use Subcommand::*;

    let search =
      Search::search(&self.search_config, &self.invocation_directory).eprint(self.color)?;

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

    for warning in &justfile.warnings {
      if self.color.stderr().active() {
        eprintln!("{:#}", warning);
      } else {
        eprintln!("{}", warning);
      }
    }

    match self.subcommand {
      Dump => self.dump(justfile),
      Run | Evaluate => self.run(justfile, &search.working_directory),
      List => self.list(justfile),
      Show { ref name } => self.show(&name, justfile),
      Summary => self.summary(justfile),
      Edit => unreachable!(),
    }
  }

  fn dump(&self, justfile: Justfile) -> Result<(), i32> {
    println!("{}", justfile);
    Ok(())
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
      Ok(status) => {
        if status.success() {
          Ok(())
        } else {
          eprintln!("Editor `{}` failed: {}", editor.to_string_lossy(), status);
          Err(status.code().unwrap_or(EXIT_FAILURE))
        }
      }
      Err(error) => {
        eprintln!(
          "Editor `{}` invocation failed: {}",
          editor.to_string_lossy(),
          error
        );
        Err(EXIT_FAILURE)
      }
    }
  }

  fn list(&self, justfile: Justfile) -> Result<(), i32> {
    // Construct a target to alias map.
    let mut recipe_aliases: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for alias in justfile.aliases.values() {
      if alias.is_private() {
        continue;
      }

      if !recipe_aliases.contains_key(alias.target.lexeme()) {
        recipe_aliases.insert(alias.target.lexeme(), vec![alias.name.lexeme()]);
      } else {
        let aliases = recipe_aliases.get_mut(alias.target.lexeme()).unwrap();
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
    println!("Available recipes:");

    for (name, recipe) in &justfile.recipes {
      if recipe.private {
        continue;
      }

      let alias_doc = format!("alias for `{}`", recipe.name);

      for (i, name) in iter::once(name)
        .chain(recipe_aliases.get(name).unwrap_or(&Vec::new()))
        .enumerate()
      {
        print!("    {}", name);
        for parameter in &recipe.parameters {
          if self.color.stdout().active() {
            print!(" {:#}", parameter);
          } else {
            print!(" {}", parameter);
          }
        }

        // Declaring this outside of the nested loops will probably be more efficient, but
        // it creates all sorts of lifetime issues with variables inside the loops.
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
          _ => print_doc(&alias_doc),
        }
        println!();
      }
    }

    Ok(())
  }

  fn run(&self, justfile: Justfile, working_directory: &Path) -> Result<(), i32> {
    if let Err(error) = InterruptHandler::install() {
      warn!("Failed to set CTRL-C handler: {}", error)
    }

    let result = justfile.run(&self, working_directory);

    if !self.quiet {
      result.eprint(self.color)
    } else {
      result.map_err(|err| err.code())
    }
  }

  fn show(&self, name: &str, justfile: Justfile) -> Result<(), i32> {
    if let Some(alias) = justfile.get_alias(name) {
      let recipe = justfile.get_recipe(alias.target.lexeme()).unwrap();
      println!("{}", alias);
      println!("{}", recipe);
      Ok(())
    } else if let Some(recipe) = justfile.get_recipe(name) {
      println!("{}", recipe);
      Ok(())
    } else {
      eprintln!("Justfile does not contain recipe `{}`.", name);
      if let Some(suggestion) = justfile.suggest(name) {
        eprintln!("Did you mean `{}`?", suggestion);
      }
      Err(EXIT_FAILURE)
    }
  }

  fn summary(&self, justfile: Justfile) -> Result<(), i32> {
    if justfile.count() == 0 {
      eprintln!("Justfile contains no recipes.");
    } else {
      let summary = justfile
        .recipes
        .iter()
        .filter(|&(_, recipe)| !recipe.private)
        .map(|(name, _)| name)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
      println!("{}", summary);
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use pretty_assertions::assert_eq;

  // This test guards against unintended changes to the argument parser. We should have
  // proper tests for all the flags, but this will do for now.
  #[test]
  fn help() {
    const EXPECTED_HELP: &str = "just v0.4.5
Casey Rodarmor <casey@rodarmor.com>
🤖 Just a command runner - https://github.com/casey/just

USAGE:
    just [FLAGS] [OPTIONS] [--] [ARGUMENTS]...

FLAGS:
        --dry-run         Print what just would do without doing it
        --dump            Print entire justfile
    -e, --edit            \
    Edit justfile with editor given by $VISUAL or $EDITOR, falling back to `vim`
        --evaluate        Print evaluated variables
        --highlight       Highlight echoed recipe lines in bold
    -l, --list            List available recipes and their arguments
        --no-highlight    Don't highlight echoed recipe lines in bold
    -q, --quiet           Suppress all output
        --summary         List names of available recipes
    -v, --verbose         Use verbose output

OPTIONS:
        --color <COLOR>
            Print colorful output [default: auto]  [possible values: auto, always, never]

    -f, --justfile <JUSTFILE>                      Use <JUSTFILE> as justfile.
        --set <VARIABLE> <VALUE>                   Set <VARIABLE> to <VALUE>
        --shell <SHELL>                            Invoke <SHELL> to run recipes [default: sh]
    -s, --show <RECIPE>                            Show information about <RECIPE>
    -d, --working-directory <WORKING-DIRECTORY>
            Use <WORKING-DIRECTORY> as working directory. --justfile must also be set


ARGS:
    <ARGUMENTS>...    The recipe(s) to run, defaults to the first recipe in the justfile";

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
      $(arguments: $arguments:expr,)?
      $(color: $color:expr,)?
      $(dry_run: $dry_run:expr,)?
      $(highlight: $highlight:expr,)?
      $(overrides: $overrides:expr,)?
      $(quiet: $quiet:expr,)?
      $(search_config: $search_config:expr,)?
      $(shell: $shell:expr,)?
      $(subcommand: $subcommand:expr,)?
      $(verbosity: $verbosity:expr,)?
    } => {
      #[test]
      fn $name() {
        let arguments = &[
          "just",
          $($arg,)*
        ];

        let want = Config {
          $(arguments: $arguments.iter().map(|argument| argument.to_string()).collect(),)?
          $(color: $color,)?
          $(dry_run: $dry_run,)?
          $(highlight: $highlight,)?
          $(
            overrides: $overrides.iter().cloned()
              .map(|(key, value): (&str, &str)| (key.to_owned(), value.to_owned())).collect(),
          )?
          $(quiet: $quiet,)?
          $(search_config: $search_config,)?
          $(shell: $shell.to_string(),)?
          $(subcommand: $subcommand,)?
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

        error(arguments);
      }
    }
  }

  fn error(arguments: &[&str]) {
    let app = Config::app();
    if let Ok(matches) = app.get_matches_from_safe(arguments) {
      Config::from_matches(&matches).expect_err("config parsing unexpectedly succeeded");
    } else {
      return;
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
    name: quiet_default,
    args: [],
    quiet: false,
  }

  test! {
    name: quiet_long,
    args: ["--quiet"],
    quiet: true,
  }

  test! {
    name: quiet_short,
    args: ["-q"],
    quiet: true,
  }

  test! {
    name: set_default,
    args: [],
    overrides: [],
  }

  test! {
    name: set_one,
    args: ["--set", "foo", "bar"],
    overrides: [("foo", "bar")],
  }

  test! {
    name: set_empty,
    args: ["--set", "foo", ""],
    overrides: [("foo", "")],
  }

  test! {
    name: set_two,
    args: ["--set", "foo", "bar", "--set", "bar", "baz"],
    overrides: [("foo", "bar"), ("bar", "baz")],
  }

  test! {
    name: set_override,
    args: ["--set", "foo", "bar", "--set", "foo", "baz"],
    overrides: [("foo", "baz")],
  }

  error! {
    name: set_bad,
    args: ["--set", "foo"],
  }

  test! {
    name: shell_default,
    args: [],
    shell: "sh",
  }

  test! {
    name: shell_set,
    args: ["--shell", "tclsh"],
    shell: "tclsh",
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
    subcommand: Subcommand::Run,
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
    subcommand: Subcommand::Evaluate,
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
    arguments: ["foo", "bar"],
  }

  test! {
    name: arguments_leading_equals,
    args: ["=foo"],
    arguments: ["=foo"],
  }

  test! {
    name: overrides,
    args: ["foo=bar", "bar=baz"],
    overrides: [("foo", "bar"), ("bar", "baz")],
  }

  test! {
    name: overrides_empty,
    args: ["foo=", "bar="],
    overrides: [("foo", ""), ("bar", "")],
  }

  test! {
    name: overrides_override_sets,
    args: ["--set", "foo", "0", "--set", "bar", "1", "foo=bar", "bar=baz"],
    overrides: [("foo", "bar"), ("bar", "baz")],
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
    arguments: ["build"],
    search_config: SearchConfig::FromSearchDirectory {
      search_directory: PathBuf::from(".."),
    },
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
    arguments: ["build"],
    search_config: SearchConfig::FromSearchDirectory {
      search_directory: PathBuf::from("foo"),
    },
  }

  error! {
    name: search_directory_conflict_justfile,
    args: ["--justfile", "bar", "foo/build"],
  }

  error! {
    name: search_directory_conflict_working_directory,
    args: ["--justfile", "bar", "--working-directory", "baz", "foo/build"],
  }
}
