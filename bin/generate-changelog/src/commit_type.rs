use crate::common::*;

#[derive(
  Deserialize, PartialEq, Serialize, Ord, Eq, PartialOrd, Copy, Clone, strum_macros::Display,
)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CommitType {
  Release,
  Breaking,
  Fixed,
  Changed,
  Added,
  Misc,
  Merge,
  Uncategorized,
}
