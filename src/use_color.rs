#[derive(Copy, Clone, Debug, PartialEq, clap::ValueEnum)]
pub(crate) enum UseColor {
  Auto,
  Always,
  Never,
}
