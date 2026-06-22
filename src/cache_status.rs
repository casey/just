use super::*;

pub(crate) enum CacheStatus {
  Hit,
  Miss(CacheEntry),
}
