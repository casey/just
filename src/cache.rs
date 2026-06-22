use super::*;

const DIR: &str = ".justcache";
const VERSION: u64 = 0;

#[derive(Serialize)]
struct Key<'a> {
  environment: &'a BTreeMap<String, Option<String>>,
  interpreter: Option<&'a Interpreter<String>>,
  lines: &'a [String],
  positional: Option<&'a [String]>,
  recipe: &'a Modulepath,
  version: u64,
  working_directory: Option<&'a Path>,
}

pub(crate) struct Cache {
  initialized: Mutex<bool>,
  path: PathBuf,
}

impl Cache {
  pub(crate) fn status(
    &self,
    environment: &BTreeMap<String, Option<String>>,
    interpreter: Option<&Interpreter<String>>,
    lines: &[String],
    positional: Option<&[String]>,
    working_directory: Option<&Path>,
    recipe: &Recipe,
  ) -> RunResult<'static, CacheStatus> {
    let key = Key {
      environment,
      interpreter,
      lines,
      positional,
      recipe: recipe.recipe_path(),
      version: VERSION,
      working_directory,
    };

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

    let len = file.metadata().map_err(context)?.len();

    if len > 0 {
      Ok(CacheStatus::Hit)
    } else {
      Ok(CacheStatus::Miss(CacheEntry { file, path }))
    }
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
}
