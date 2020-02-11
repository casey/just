use crate::common::*;

pub(crate) struct LoadError<'path> {
  pub(crate) path:     &'path Path,
  pub(crate) io_error: io::Error,
}

impl Error for LoadError<'_> {}

impl Display for LoadError<'_> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "Failed to read justfile at `{}`: {}",
      self.path.display(),
      self.io_error
    )
  }
}
