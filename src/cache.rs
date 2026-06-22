use super::*;

const DIR: &str = ".justcache";

pub(crate) struct Cache {
  initialized: Mutex<bool>,
  path: PathBuf,
}

impl Cache {
  pub(crate) fn status(
    &self,
    key: CacheKey,
    outputs: &[PathBuf],
  ) -> RunResult<'static, CacheStatus> {
    let mut hasher = blake3::Hasher::new();

    serde_json::to_writer(&mut hasher, &key)
      .map_err(|source| Error::CacheKeySerialize { source })?;

    let hash = hasher.finalize();

    let path = self.entry(hash)?;

    let context = |source| Error::FilesystemIo {
      path: path.clone(),
      source,
    };

    let file = fs::OpenOptions::new()
      .create(true)
      .read(true)
      .truncate(false)
      .write(true)
      .open(&path)
      .map_err(context)?;

    file.lock().map_err(context)?;

    if file.metadata().map_err(context)?.len() == 0 {
      return Ok(CacheStatus::Miss(CacheEntry { file, path }));
    }

    for output in outputs {
      if !filesystem::exists(output)? {
        return Ok(CacheStatus::Miss(CacheEntry { file, path }));
      }
    }

    Ok(CacheStatus::Hit)
  }

  pub(crate) fn new(search: &Search) -> Self {
    Self {
      path: search.justfile_parent().join(DIR),
      initialized: Mutex::new(false),
    }
  }

  fn entry(&self, key: blake3::Hash) -> RunResult<'static, PathBuf> {
    let mut initialized = self.initialized.lock().unwrap();

    if !*initialized {
      fs::create_dir_all(&self.path).map_err(|source| Error::FilesystemIo {
        source,
        path: self.path.clone(),
      })?;
      *initialized = true;
    }

    Ok(self.path.join(format!("{key}.json")))
  }

  pub(crate) fn inputs(
    context: &ExecutionContext,
    value: Value,
    working_directory: Option<&Path>,
  ) -> RunResult<'static, BTreeMap<String, blake3::Hash>> {
    let base = match working_directory {
      Some(working_directory) => working_directory.to_owned(),
      None => context.working_directory(),
    };

    let mut inputs = BTreeMap::new();

    for input in value.elements() {
      let path = base.join(input);

      let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
          return Err(Error::CacheInputMissing { path });
        }
        Err(source) => return Err(Error::FilesystemIo { source, path }),
      };

      if metadata.is_dir() {
        return Err(Error::CacheInputDirectory { path });
      }

      let mut hasher = blake3::Hasher::new();

      hasher
        .update_mmap_rayon(&path)
        .map_err(|source| Error::FilesystemIo {
          source,
          path: path.clone(),
        })?;

      inputs.insert(input.into(), hasher.finalize());
    }

    Ok(inputs)
  }

  pub(crate) fn outputs(
    context: &ExecutionContext,
    value: Value,
    working_directory: Option<&Path>,
  ) -> Vec<PathBuf> {
    let base = match working_directory {
      Some(working_directory) => working_directory.to_owned(),
      None => context.working_directory(),
    };

    value
      .elements()
      .iter()
      .map(|output| base.join(output))
      .collect()
  }
}
