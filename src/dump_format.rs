use super::*;

#[derive(Debug, Default, PartialEq, Clone, ValueEnum)]
pub(crate) enum DumpFormat {
  Json,
  #[default]
  Just,
}
