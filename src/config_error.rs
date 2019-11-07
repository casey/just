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
}

impl ConfigError {
  pub(crate) fn internal(message: impl Into<String>) -> ConfigError {
    ConfigError::Internal {
      message: message.into(),
    }
  }
}

impl Error for ConfigError {}
