#[derive(Debug, PartialEq, Default)]
pub(crate) enum DumpFormat {
  Json,
  #[default]
  Just,
}
