use super::*;

#[derive(Deserialize, Serialize)]
pub(crate) struct CacheEntry {
  #[serde(default)]
  pub(crate) outputs: BTreeMap<String, CacheOutput>,
  pub(crate) recipe: Modulepath,
}

impl CacheEntry {
  pub(crate) fn load(file: &File, path: &Utf8Path) -> RunResult<'static, Option<Self>> {
    let json = io::read_to_string(file).map_err(|source| Error::FilesystemIo {
      source,
      path: path.into(),
    })?;

    if json.is_empty() {
      return Ok(None);
    }

    let entry = serde_json::from_str(&json).map_err(|source| Error::CacheEntryRead {
      source,
      path: path.into(),
    })?;

    Ok(Some(entry))
  }
}
