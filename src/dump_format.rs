#[derive(Debug, PartialEq, Clone, clap::ValueEnum)]
pub(crate) enum DumpFormat {
  Json,
  Just,
}
