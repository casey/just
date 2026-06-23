use super::*;

#[derive(Deserialize, Serialize)]
pub(crate) struct CacheEntry {
  pub(crate) recipe: Modulepath,
}
