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
  fn parse_modulepath(values: &[String]) -> ConfigResult<Modulepath> {
    let path = values.iter().map(String::as_str).collect::<Vec<&str>>();

    let path = if path.len() == 1 && path[0].contains(' ') {
      path[0].split_whitespace().collect::<Vec<&str>>()
    } else {
      path
    };

    path
      .as_slice()
      .try_into()
      .map_err(|()| ConfigError::ModulePath {
        path: values.to_vec(),
      })
  }

  fn search_config(arguments: &Arguments, positional: &Positional) -> ConfigResult<SearchConfig> {
    if arguments.global_justfile {
      return Ok(SearchConfig::GlobalJustfile);
    }

    let justfile = arguments.justfile.clone();

    let working_directory = arguments.working_directory.clone();

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

  pub(crate) fn timestamp(&self) -> Option<String> {
    self.timestamp.then(|| {
      chrono::Local::now()
        .format(&self.timestamp_format)
        .to_string()
    })
  }

  fn parse_override(path: &str) -> ConfigResult<(Modulepath, String)> {
    let mut path = Modulepath::try_from([path].as_slice())
      .map_err(|()| ConfigError::OverridePath { path: path.into() })?;

    let name = path.components.pop().unwrap();

    Ok((path, name))
  }

  fn subcommand(arguments: &Arguments, positional: &Positional) -> ConfigResult<Subcommand> {
    if arguments.subcommand.changelog {
      Ok(Subcommand::Changelog)
    } else if arguments.subcommand.choose {
      Ok(Subcommand::Choose {
        chooser: arguments.chooser.clone(),
      })
    } else if let Some(mut command) = arguments.subcommand.command.clone() {
      Ok(Subcommand::Command {
        binary: command.remove(0),
        arguments: command,
      })
    } else if let Some(shell) = arguments.subcommand.completions {
      Ok(Subcommand::Completions { shell })
    } else if arguments.subcommand.dump {
      Ok(Subcommand::Dump {
        format: arguments.dump_format,
      })
    } else if arguments.subcommand.edit {
      Ok(Subcommand::Edit)
    } else if arguments.subcommand.evaluate {
      let path = if positional.arguments.is_empty() {
        Modulepath::default()
      } else if positional.arguments.len() == 1 {
        Self::parse_modulepath(&positional.arguments)?
      } else {
        return Err(ConfigError::SubcommandArguments {
          subcommand: "EVALUATE",
          arguments: positional.arguments.iter().skip(1).cloned().collect(),
        });
      };

      Ok(Subcommand::Evaluate {
        format: arguments.evaluate_format,
        path,
      })
    } else if arguments.subcommand.fmt {
      Ok(Subcommand::Format)
    } else if arguments.subcommand.groups {
      Ok(Subcommand::Groups)
    } else if arguments.subcommand.init {
      Ok(Subcommand::Init)
    } else if arguments.subcommand.json {
      Ok(Subcommand::Dump {
        format: DumpFormat::Json,
      })
    } else if let Some(path) = arguments.subcommand.list.as_deref() {
      Ok(Subcommand::List {
        path: Self::parse_modulepath(path)?,
      })
    } else if arguments.subcommand.man {
      Ok(Subcommand::Man)
    } else if let Some(request) = arguments.subcommand.request.as_deref() {
      Ok(Subcommand::Request {
        request: serde_json::from_str(request)
          .map_err(|source| ConfigError::RequestParse { source })?,
      })
    } else if let Some(path) = arguments.subcommand.show.as_deref() {
      Ok(Subcommand::Show {
        path: Self::parse_modulepath(path)?,
      })
    } else if arguments.subcommand.summary {
      Ok(Subcommand::Summary)
    } else if let Some(path) = arguments.subcommand.usage.as_deref() {
      Ok(Subcommand::Usage {
        path: Self::parse_modulepath(path)?,
      })
    } else if arguments.subcommand.variables {
      Ok(Subcommand::Variables)
    } else {
      Ok(Subcommand::Run {
        arguments: positional.arguments.clone(),
      })
    }
  }

  pub(crate) fn from_arguments(arguments: Arguments) -> ConfigResult<Self> {
    let mut overrides = BTreeMap::new();
    let mut values = arguments.set.iter();
    while let Some(path) = values.next() {
      overrides.insert(
        Self::parse_override(path)?,
        values
          .next()
          .ok_or_else(|| ConfigError::internal(format!("--set for `{path}` did not have value")))?
          .into(),
      );
    }

    let positional = Positional::from_values(Some(arguments.arguments.iter().map(String::as_str)));

    for (path, value) in &positional.overrides {
      overrides.insert(Self::parse_override(path)?, value.into());
    }

    let search_config = Self::search_config(&arguments, &positional)?;

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

    let subcommand = Self::subcommand(&arguments, &positional)?;

    if !subcommand.takes_arguments() {
      match (!overrides.is_empty(), !positional.arguments.is_empty()) {
        (false, false) => {}
        (true, false) => {
          return Err(ConfigError::SubcommandOverrides {
            subcommand: subcommand.name(),
            overrides: format_overrides(),
          });
        }
        (false, true) => {
          return Err(ConfigError::SubcommandArguments {
            arguments: positional.arguments,
            subcommand: subcommand.name(),
          });
        }
        (true, true) => {
          return Err(ConfigError::SubcommandOverridesAndArguments {
            arguments: positional.arguments,
            subcommand: subcommand.name(),
            overrides: format_overrides(),
          });
        }
      }
    }

    let unstable = arguments.unstable || subcommand == Subcommand::Summary;
    let color = Color::new(arguments.indentation, arguments.color);

    let invocation_directory = env::current_dir().context(config_error::CurrentDir)?;

    Self::warn_non_unicode_path(color, "invocation directory", &invocation_directory);

    Ok(Self {
      alias_style: arguments.alias_style,
      allow_missing: arguments.allow_missing,
      ceiling: arguments.ceiling,
      check: arguments.check,
      color,
      command_color: arguments.command_color.map(CommandColor::into),
      cygpath: arguments.cygpath,
      dotenv_filename: arguments.dotenv_filename,
      dotenv_path: arguments.dotenv_path,
      dry_run: arguments.dry_run,
      explain: arguments.explain,
      highlight: !arguments.no_highlight,
      invocation_directory,
      groups: arguments.group,
      list_heading: arguments.list_heading,
      list_prefix: arguments.list_prefix,
      list_submodules: arguments.list_submodules,
      load_dotenv: !arguments.no_dotenv,
      no_aliases: arguments.no_aliases,
      no_dependencies: arguments.no_deps,
      one: arguments.one,
      overrides,
      search_config,
      shell: arguments.shell,
      shell_args: if arguments.clear_shell_args {
        Some(Vec::new())
      } else if arguments.shell_arg.is_empty() {
        None
      } else {
        Some(arguments.shell_arg)
      },
      shell_command: arguments.shell_command,
      subcommand,
      tempdir: arguments.tempdir,
      timestamp: arguments.timestamp,
      timestamp_format: arguments.timestamp_format,
      unsorted: arguments.unsorted,
      unstable,
      verbosity: if arguments.quiet {
        Verbosity::Quiet
      } else {
        Verbosity::from_flag_occurrences(arguments.verbose)
      },
      yes: arguments.yes,
    })
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

  pub(crate) fn warn_non_unicode_path(color: Color, name: &str, path: &Path) {
    if path.to_str().is_none() {
      eprintln!(
        "{}The {name} path `{}` is not Unicode. Just is considering phasing-out support for \
        non-Unicode paths. If you see this warning, please leave a comment on
        https://github.com/casey/just/issues/3229. Thank you!{}",
        path.display(),
        color.warning().prefix(),
        color.warning().suffix(),
      );
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
    let arguments = Arguments::try_parse_from(arguments).expect("argument parsing failed");
    let have = Config::from_arguments(arguments).expect("config parsing failed");
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

        Arguments::try_parse_from(arguments).expect_err("Expected clap error");
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

        let arguments =
          Arguments::try_parse_from(arguments).expect("Matching fails");

        match Config::from_arguments(arguments).expect_err("config parsing succeeded") {
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

        match Arguments::try_parse_from(arguments) {
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
    subcommand: Subcommand::Completions{ shell: Shell::Bash },
  }

  test! {
    name: subcommand_completions_uppercase,
    args: ["--completions", "BASH"],
    subcommand: Subcommand::Completions{ shell: Shell::Bash },
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
      format: EvaluateFormat::Just,
      path: Modulepath::default(),
    },
  }

  test! {
    name: subcommand_evaluate_overrides,
    args: ["--evaluate", "x=y"],
    overrides: map!{"x": "y"},
    subcommand: Subcommand::Evaluate {
      format: EvaluateFormat::Just,
      path: Modulepath::default(),
    },
  }

  test! {
    name: subcommand_evaluate_overrides_with_argument,
    args: ["--evaluate", "x=y", "foo"],
    overrides: map!{"x": "y"},
    subcommand: Subcommand::Evaluate {
      format: EvaluateFormat::Just,
      path: Modulepath::try_from(["foo"].as_slice()).unwrap(),
    },
  }

  test! {
    name: subcommand_list_long,
    args: ["--list"],
    subcommand: Subcommand::List { path: Modulepath::default() },
  }

  test! {
    name: subcommand_list_short,
    args: ["-l"],
    subcommand: Subcommand::List { path: Modulepath::default() },
  }

  test! {
    name: subcommand_list_arguments,
    args: ["--list", "bar"],
    subcommand: Subcommand::List { path: Modulepath::try_from(["bar"].as_slice()).unwrap() },
  }

  test! {
    name: subcommand_show_long,
    args: ["--show", "build"],
    subcommand: Subcommand::Show { path: Modulepath::try_from(["build"].as_slice()).unwrap() },
  }

  test! {
    name: subcommand_show_short,
    args: ["-s", "build"],
    subcommand: Subcommand::Show { path: Modulepath::try_from(["build"].as_slice()).unwrap() },
  }

  test! {
    name: subcommand_show_multiple_args,
    args: ["--show", "foo", "bar"],
    subcommand: Subcommand::Show {
      path: Modulepath::try_from(["foo", "bar"].as_slice()).unwrap(),
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
