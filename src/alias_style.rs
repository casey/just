use super::*;

#[derive(Debug, Default, PartialEq, Clone, ValueEnum)]
pub(crate) enum AliasStyle {
  Left,
  #[default]
  Right,
  Separate,
}
