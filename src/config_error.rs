use crate::common::*;

pub(crate) enum ConfigError {
  Internal { message: String },
}

impl Display for ConfigError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use ConfigError::*;

    match self {
      Internal { message } => write!(
        f,
        "Internal config error, this may indicate a bug in just: {} \
         consider filing an issue: https://github.com/casey/just/issues/new",
        message
      ),
    }
  }
}
