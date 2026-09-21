use super::*;

pub(crate) trait PathBufExt {
  fn into_utf8(self) -> Result<Utf8PathBuf, FromPathBufError>;
}

impl PathBufExt for std::path::PathBuf {
  fn into_utf8(self) -> Result<Utf8PathBuf, FromPathBufError> {
    Utf8PathBuf::try_from(self)
  }
}
