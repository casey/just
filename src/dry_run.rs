use super::*;

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum DryRun {
  #[default]
  All,
  Recipes,
}
