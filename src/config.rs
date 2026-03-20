use super::*;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Config {
  pub(crate) alias_style: AliasStyle,
  pub(crate) allow_missing: bool,
  pub(crate) ceiling: Option<PathBuf>,
  pub(crate) check: bool,
  pub(crate) color: Color,
  pub(crate) command_color: Option<ansi_term::Color>,
  pub(crate) cygpath: PathBuf,
  pub(crate) dotenv_filename: Option<String>,
  pub(crate) dotenv_path: Option<PathBuf>,
  pub(crate) dry_run: bool,
  pub(crate) explain: bool,
  pub(crate) groups: Vec<String>,
  pub(crate) highlight: bool,
  pub(crate) invocation_directory: PathBuf,
  pub(crate) list_heading: String,
  pub(crate) list_prefix: String,
  pub(crate) list_submodules: bool,
  pub(crate) load_dotenv: bool,
  pub(crate) no_aliases: bool,
  pub(crate) no_dependencies: bool,
  pub(crate) one: bool,
  pub(crate) overrides: BTreeMap<(Modulepath, String), String>,
  pub(crate) search_config: SearchConfig,
  pub(crate) shell: Option<String>,
  pub(crate) shell_args: Option<Vec<String>>,
  pub(crate) shell_command: bool,
  pub(crate) subcommand: Subcommand,
  pub(crate) tempdir: Option<PathBuf>,
  pub(crate) timestamp: bool,
  pub(crate) timestamp_format: String,
  pub(crate) unsorted: bool,
  pub(crate) unstable: bool,
  pub(crate) verbosity: Verbosity,
  pub(crate) yes: bool,
}

impl Config {
  pub(crate) fn timestamp(&self) -> Option<String> {
    self.timestamp.then(|| {
      chrono::Local::now()
        .format(&self.timestamp_format)
        .to_string()
    })
  }

  pub(crate) fn parse_override(path: &str) -> ConfigResult<(Modulepath, String)> {
    let mut path = Modulepath::try_from([path].as_slice())
      .map_err(|()| ConfigError::OverridePath { path: path.into() })?;

    let name = path.path.pop().unwrap();

    Ok((path, name))
  }

  pub(crate) fn require_unstable(
    &self,
    justfile: &Justfile,
    unstable_feature: UnstableFeature,
  ) -> RunResult<'static> {
    if self.unstable || justfile.settings.unstable {
      Ok(())
    } else {
      Err(Error::UnstableFeature { unstable_feature })
    }
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    clap::error::{ContextKind, ContextValue},
    pretty_assertions::assert_eq,
  };

  macro_rules! test {
    {
      name: $name:ident,
      args: [$($arg:expr),*],
      $(color: $color:expr,)?
      $(dry_run: $dry_run:expr,)?
      $(dump_format: $dump_format:expr,)?
      $(highlight: $highlight:expr,)?
      $(no_dependencies: $no_dependencies:expr,)?
      $(overrides: $overrides:expr,)?
      $(search_config: $search_config:expr,)?
      $(shell: $shell:expr,)?
      $(shell_args: $shell_args:expr,)?
      $(subcommand: $subcommand:expr,)?
      $(unsorted: $unsorted:expr,)?
      $(unstable: $unstable:expr,)?
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
          $(overrides: $overrides,)?
          $(search_config: $search_config,)?
          $(shell: $shell,)?
          $(shell_args: $shell_args,)?
          $(subcommand: $subcommand,)?
          $(unsorted: $unsorted,)?
          $(unstable: $unstable,)?
          $(verbosity: $verbosity,)?
          ..testing::config(&[])
        };

        test(arguments, want);
      }
    }
  }

  #[track_caller]
  fn test(arguments: &[&str], want: Config) {
    let args = Arguments::try_parse_from_args(arguments).expect("argument parsing failed");
    let have = args.into_config().expect("config parsing failed");
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

        Arguments::try_parse_from_args(arguments).expect_err("Expected clap error");
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

        let args = Arguments::try_parse_from_args(arguments).expect("Matching fails");

        match args.into_config().expect_err("config parsing succeeded") {
          $error => { $($check)? }
          other => panic!("Unexpected config error: {other}"),
        }
      }
    }
  }

  macro_rules! error_matches {
    (
      name: $name:ident,
      args: [$($arg:expr),*],
      error: $error:pat,
      $(check: $check:block,)?
    ) => {
      #[test]
      fn $name() {
        let arguments = &[
          "just",
          $($arg,)*
        ];

        match Arguments::try_parse_from_args(arguments) {
          Err($error) => { $($check)? }
          other => panic!("Unexpected result from get matches: {other:?}")
        }
      }
    };
  }

  macro_rules! map {
    {} => {
      BTreeMap::new()
    };
    {
      $($key:literal : $value:literal),* $(,)?
    } => {
      {
        let mut map: BTreeMap<(Modulepath, String), String> = BTreeMap::new();
        $(
          map.insert((Modulepath::default(), $key.to_owned()), $value.to_owned());
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
    overrides: map!(),
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
    },
  }

  test! {
    name: set_one,
    args: ["--set", "foo", "bar"],
    overrides: map!{"foo": "bar"},
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
    },
  }

  test! {
    name: set_empty,
    args: ["--set", "foo", ""],
    overrides: map!{"foo": ""},
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
    },
  }

  test! {
    name: set_two,
    args: ["--set", "foo", "bar", "--set", "bar", "baz"],
    overrides: map!{"foo": "bar", "bar": "baz"},
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
    },
  }

  test! {
    name: set_override,
    args: ["--set", "foo", "bar", "--set", "foo", "baz"],
    overrides: map!{"foo": "baz"},
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
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
    overrides: map!{},
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
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
    subcommand: Subcommand::Completions{ shell: completions::Shell::Bash },
  }

  test! {
    name: subcommand_completions_uppercase,
    args: ["--completions", "BASH"],
    subcommand: Subcommand::Completions{ shell: completions::Shell::Bash },
  }

  error! {
    name: subcommand_completions_invalid,
    args: ["--completions", "monstersh"],
  }

  test! {
    name: subcommand_dump,
    args: ["--dump"],
    subcommand: Subcommand::Dump { format: DumpFormat::Just },
  }

  test! {
    name: subcommand_json,
    args: ["--json"],
    subcommand: Subcommand::Dump { format: DumpFormat::Json },
  }

  test! {
    name: subcommand_edit,
    args: ["--edit"],
    subcommand: Subcommand::Edit,
  }

  test! {
    name: subcommand_evaluate,
    args: ["--evaluate"],
    overrides: map!{},
    subcommand: Subcommand::Evaluate {
      variable: None,
    },
  }

  test! {
    name: subcommand_evaluate_overrides,
    args: ["--evaluate", "x=y"],
    overrides: map!{"x": "y"},
    subcommand: Subcommand::Evaluate {
      variable: None,
    },
  }

  test! {
    name: subcommand_evaluate_overrides_with_argument,
    args: ["--evaluate", "x=y", "foo"],
    overrides: map!{"x": "y"},
    subcommand: Subcommand::Evaluate {
      variable: Some("foo".to_owned()),
    },
  }

  test! {
    name: subcommand_list_long,
    args: ["--list"],
    subcommand: Subcommand::List{ path: Modulepath { path: Vec::new(), spaced: false } },
  }

  test! {
    name: subcommand_list_short,
    args: ["-l"],
    subcommand: Subcommand::List{ path: Modulepath { path: Vec::new(), spaced: false } },
  }

  test! {
    name: subcommand_list_arguments,
    args: ["--list", "bar"],
    subcommand: Subcommand::List{ path: Modulepath { path: vec!["bar".into()], spaced: false } },
  }

  test! {
    name: subcommand_show_long,
    args: ["--show", "build"],
    subcommand: Subcommand::Show { path: Modulepath { path: vec!["build".into()], spaced: false } },
  }

  test! {
    name: subcommand_show_short,
    args: ["-s", "build"],
    subcommand: Subcommand::Show { path: Modulepath { path: vec!["build".into()], spaced: false } },
  }

  test! {
    name: subcommand_show_multiple_args,
    args: ["--show", "foo", "bar"],
    subcommand: Subcommand::Show {
      path: Modulepath {
        path: vec!["foo".into(), "bar".into()],
        spaced: true,
      },
    },
  }

  test! {
    name: subcommand_summary,
    args: ["--summary"],
    subcommand: Subcommand::Summary,
    unstable: true,
  }

  test! {
    name: arguments,
    args: ["foo", "bar"],
    overrides: map!{},
    subcommand: Subcommand::Run {
      arguments: vec![String::from("foo"), String::from("bar")],
    },
  }

  test! {
    name: overrides,
    args: ["foo=bar", "bar=baz"],
    overrides: map!{"foo": "bar", "bar": "baz"},
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
    },
  }

  test! {
    name: overrides_empty,
    args: ["foo=", "bar="],
    overrides: map!{"foo": "", "bar": ""},
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
    },
  }

  test! {
    name: overrides_override_sets,
    args: ["--set", "foo", "0", "--set", "bar", "1", "foo=bar", "bar=baz"],
    overrides: map!{"foo": "bar", "bar": "baz"},
    subcommand: Subcommand::Run {
      arguments: Vec::new(),
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
    shell_args: Some(Vec::new()),

  }

  test! {
    name: shell_args_clear_and_set,
    args: ["--clear-shell-args", "--shell-arg", "bar"],
    shell_args: Some(vec!["bar".to_owned()]),

  }

  test! {
    name: shell_args_set_and_clear,
    args: ["--shell-arg", "bar", "--clear-shell-args"],
    shell_args: Some(Vec::new()),

  }

  test! {
    name: shell_args_set_multiple_and_clear,
    args: ["--shell-arg", "bar", "--shell-arg", "baz", "--clear-shell-args"],
    shell_args: Some(Vec::new()),

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
    subcommand: Subcommand::Run { arguments: vec!["build".to_owned()] },
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
    subcommand: Subcommand::Run { arguments: vec!["build".to_owned()] },
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

  error_matches! {
    name: completions_argument,
    args: ["--completions", "foo"],
    error: error,
    check: {
      assert_eq!(error.kind(), clap::error::ErrorKind::InvalidValue);
      assert_eq!(error.context().collect::<Vec<_>>(), vec![
        (
          ContextKind::InvalidArg,
          &ContextValue::String("--completions <SHELL>".into())),
        (
          ContextKind::InvalidValue,
          &ContextValue::String("foo".into()),
        ),
        (
          ContextKind::ValidValue,
          &ContextValue::Strings([
            "bash".into(),
            "elvish".into(),
            "fish".into(),
            "nushell".into(),
            "powershell".into(),
            "zsh".into()].into()
          ),
        ),
      ]);
    },
  }

  error! {
    name: changelog_arguments,
    args: ["--changelog", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, "CHANGELOG");
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: dump_arguments,
    args: ["--dump", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, "DUMP");
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: edit_arguments,
    args: ["--edit", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, "EDIT");
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: fmt_arguments,
    args: ["--fmt", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, "FORMAT");
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: fmt_alias,
    args: ["--format", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, "FORMAT");
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: init_arguments,
    args: ["--init", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, "INIT");
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: init_alias,
    args: ["--initialize", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, "INIT");
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: summary_arguments,
    args: ["--summary", "bar"],
    error: ConfigError::SubcommandArguments { subcommand, arguments },
    check: {
      assert_eq!(subcommand, "SUMMARY");
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: subcommand_overrides_and_arguments,
    args: ["--summary", "bar=baz", "bar"],
    error: ConfigError::SubcommandOverridesAndArguments { subcommand, arguments, overrides },
    check: {
      assert_eq!(subcommand, "SUMMARY");
      assert_eq!(overrides, vec!["bar=baz"]);
      assert_eq!(arguments, &["bar"]);
    },
  }

  error! {
    name: summary_overrides,
    args: ["--summary", "bar=baz"],
    error: ConfigError::SubcommandOverrides { subcommand, overrides },
    check: {
      assert_eq!(subcommand, "SUMMARY");
      assert_eq!(overrides, vec!["bar=baz"]);
    },
  }
}
