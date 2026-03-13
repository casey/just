use super::*;

#[derive(Copy, Clone, Debug, PartialEq, ValueEnum)]
pub(crate) enum UseColor {
  Always,
  Auto,
  Never,
}
