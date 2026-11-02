use super::*;

const DIR: &str = ".justcache";

pub(crate) struct Cache {
  initialized: Mutex<bool>,
  path: Utf8PathBuf,
}

impl Cache {
  pub(crate) fn status(
    &self,
    config: &Config,
    key: CacheKey,
    outputs: BTreeMap<String, Utf8PathBuf>,
  ) -> RunResult<'static, CacheStatus> {
    let hash = Hash::json(&key).map_err(|source| Error::CacheKeySerialize { source })?;

    let path = self.entry(hash)?;

    if config.verbosity.grandiloquent() {
      let json =
        serde_json::to_string_pretty(&key).map_err(|source| Error::CacheKeySerialize { source })?;
      let stderr = config.color.stderr();
      eprintln!(
        "{}",
        stderr.banner().paint(&format!("===> cache key {hash}:")),
      );
      eprintln!("{}", stderr.doc().paint(&json));
    }

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

    let Some(entry) = CacheEntry::load(&file, &path)? else {
      return Ok(CacheStatus::Miss(CacheLock {
        file,
        outputs,
        path,
        recipe: key.recipe.clone(),
      }));
    };

    let expected = entry
      .outputs
      .into_iter()
      .map(|(name, output)| (name, Some(output)))
      .collect();

    if Self::outputs(&outputs)? == expected {
      Ok(CacheStatus::Hit)
    } else {
      Ok(CacheStatus::Miss(CacheLock {
        file,
        outputs,
        path,
        recipe: key.recipe.clone(),
      }))
    }
  }

  pub(crate) fn new(search: &Search) -> Self {
    Self {
      path: Self::dir(search),
      initialized: Mutex::new(false),
    }
  }

  fn entry(&self, key: Hash) -> RunResult<'static, Utf8PathBuf> {
    let mut initialized = self.initialized.lock().unwrap();

    if !*initialized {
      filesystem::create_dir_all(&self.path)?;
      *initialized = true;
    }

    Ok(self.path.join(format!("{key}.json")))
  }

  pub(crate) fn inputs(
    value: Value,
    working_directory: &Utf8Path,
  ) -> RunResult<'static, BTreeMap<String, Hash>> {
    let mut inputs = BTreeMap::new();

    for input in value {
      let path = working_directory.join(&input);

      let Some(metadata) = filesystem::metadata_opt(&path)? else {
        return Err(Error::CacheInputMissing { path });
      };

      if metadata.is_dir() {
        return Err(Error::CacheInputDirectory { path });
      }

      inputs.insert(input, Hash::file(&path)?);
    }

    Ok(inputs)
  }

  pub(crate) fn outputs(
    paths: &BTreeMap<String, Utf8PathBuf>,
  ) -> RunResult<'static, BTreeMap<String, Option<CacheOutput>>> {
    let mut outputs = BTreeMap::new();

    for (name, path) in paths {
      let output = match filesystem::metadata_opt(path)? {
        Some(metadata) => {
          if metadata.is_dir() {
            Some(CacheOutput::Directory)
          } else {
            Some(CacheOutput::File(Hash::file(path)?))
          }
        }
        None => None,
      };

      outputs.insert(name.clone(), output);
    }

    Ok(outputs)
  }

  pub(crate) fn dir(search: &Search) -> Utf8PathBuf {
    search.justfile_parent().join(DIR)
  }
}
