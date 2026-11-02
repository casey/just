use super::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Hash(blake3::Hash);

impl Hash {
  pub(crate) fn file(path: &Utf8Path) -> RunResult<'static, Self> {
    let mut hasher = blake3::Hasher::new();

    hasher
      .update_mmap_rayon(path)
      .map_err(|source| Error::FilesystemIo {
        source,
        path: path.into(),
      })?;

    Ok(Self(hasher.finalize()))
  }

  pub(crate) fn json(value: &impl Serialize) -> serde_json::Result<Self> {
    let mut hasher = blake3::Hasher::new();

    serde_json::to_writer(&mut hasher, value)?;

    Ok(Self(hasher.finalize()))
  }
}

impl Display for Hash {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Serialize for Hash {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl<'de> Deserialize<'de> for Hash {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let string = String::deserialize(deserializer)?;

    match blake3::Hash::from_hex(&string) {
      Ok(hash) => Ok(Self(hash)),
      Err(_) => Err(serde::de::Error::custom(format!(
        "invalid hash: `{string}`"
      ))),
    }
  }
}
