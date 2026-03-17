use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub(crate) enum DumpFormat {
  Json,
  #[default]
  Just,
}
