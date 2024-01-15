#[derive(Debug, PartialEq, Default, Clone)]
pub(crate) enum DumpFormat {
  Json,
  #[default]
  Just,
}
