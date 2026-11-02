use super::*;

pub(crate) struct CacheLock {
  pub(crate) file: File,
  pub(crate) outputs: BTreeMap<String, Utf8PathBuf>,
  pub(crate) path: Utf8PathBuf,
  pub(crate) recipe: Modulepath,
}

impl CacheLock {
  pub(crate) fn save(mut self, recipe: &str) -> RunResult<'_> {
    let outputs = Cache::outputs(&self.outputs)?
      .into_iter()
      .map(|(name, output)| match output {
        Some(output) => Ok((name, output)),
        None => Err(Error::CacheOutputMissing {
          recipe,
          output: name,
        }),
      })
      .collect::<RunResult<BTreeMap<String, CacheOutput>>>()?;

    let context = |source| Error::FilesystemIo {
      source,
      path: self.path.clone(),
    };

    self.file.set_len(0).map_err(context)?;

    self.file.rewind().map_err(context)?;

    let entry = CacheEntry {
      outputs,
      recipe: self.recipe,
    };

    serde_json::to_writer(&mut self.file, &entry).map_err(|source| Error::CacheEntryWrite {
      source,
      path: self.path,
    })
  }
}
