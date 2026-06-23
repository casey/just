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

    serde_json::to_writer(&mut self.file, &entry).map_err(|source| Error::CacheEntryWrite {
      source,
      path: self.path,
    })
  }
}
