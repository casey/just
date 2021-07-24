use crate::common::*;

#[derive(Debug)]
pub(crate) struct LoadError {
  pub(crate) path:     PathBuf,
  pub(crate) io_error: io::Error,
}

impl Error for LoadError {}

impl std::error::Error for LoadError {}

impl Display for LoadError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "Failed to read justfile at `{}`: {}",
      self.path.display(),
      self.io_error
    )
  }
}
