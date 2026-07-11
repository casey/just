use super::*;

pub(crate) struct CacheLock {
  pub(crate) file: File,
  pub(crate) path: PathBuf,
  pub(crate) recipe: Modulepath,
}

impl CacheLock {
  pub(crate) fn save(mut self) -> RunResult<'static> {
    let entry = CacheEntry {
      recipe: self.recipe,
    };

    self
      .file
      .set_len(0)
      .and_then(|()| self.file.rewind())
      .map_err(|source| Error::FilesystemIo {
        source,
        path: self.path.clone(),
      })?;

    serde_json::to_writer(&mut self.file, &entry).map_err(|source| Error::CacheEntryWrite {
      source,
      path: self.path,
    })
  }
}
