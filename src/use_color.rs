use super::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, ValueEnum)]
pub(crate) enum UseColor {
  Always,
  #[default]
  Auto,
  Never,
}
