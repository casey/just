use crate::common::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum ConfigError {
  #[snafu(display(
    "Internal config error, this may indicate a bug in just: {} \
     consider filing an issue: https://github.com/casey/just/issues/new",
    message
  ))]
  Internal { message: String },
  #[snafu(display("Could not canonicalize justfile path `{}`: {}", path.display(), source))]
  JustfilePathCanonicalize { path: PathBuf, source: io::Error },
  #[snafu(display("Failed to get current directory: {}", source))]
  CurrentDir { source: io::Error },
  #[snafu(display(
    "Path-prefixed recipes may not be used with `--working-directory` or `--justfile`."
  ))]
  SearchDirConflict,
  #[snafu(display(
    "`{}` used with unexpected arguments: {}",
    subcommand,
    List::and_ticked(arguments)
  ))]
  SubcommandArguments {
    subcommand: String,
    arguments: Vec<String>,
  },
  #[snafu(display(
      "`{}` used with unexpected overrides: {}; and arguments: {}",
      subcommand,
      List::and_ticked(overrides.iter().map(|(key, value)| format!("{}={}", key, value))),
      List::and_ticked(arguments)))
  ]
  SubcommandOverridesAndArguments {
    subcommand: String,
    overrides: BTreeMap<String, String>,
    arguments: Vec<String>,
  },
  #[snafu(display(
      "`{}` used with unexpected overrides: {}",
      subcommand,
      List::and_ticked(overrides.iter().map(|(key, value)| format!("{}={}", key, value))),
  ))]
  SubcommandOverrides {
    subcommand: String,
    overrides: BTreeMap<String, String>,
  },
}

impl ConfigError {
  pub(crate) fn internal(message: impl Into<String>) -> ConfigError {
    ConfigError::Internal {
      message: message.into(),
    }
  }
}

impl Error for ConfigError {}
