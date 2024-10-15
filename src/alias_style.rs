use super::*;

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub(crate) enum AliasStyle {
  Inline,
  InlineLeft,
  Recipe,
}
