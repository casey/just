use super::*;

pub(crate) struct CacheEntry {
  pub(crate) file: File,
  pub(crate) path: PathBuf,
}

impl CacheEntry {
  pub(crate) fn save(mut self) -> RunResult<'static> {
    self
      .file
      .write_all(b"{}")
      .map_err(|source| Error::FilesystemIo {
        source,
        path: self.path,
      })
  }
}
