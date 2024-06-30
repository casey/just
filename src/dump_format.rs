use super::*;

#[derive(Debug, PartialEq, Clone, ValueEnum)]
pub(crate) enum DumpFormat {
  Json,
  Just,
}
