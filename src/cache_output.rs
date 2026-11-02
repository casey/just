use super::*;

#[derive(Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CacheOutput {
  Directory,
  File(blake3::Hash),
}
