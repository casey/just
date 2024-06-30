use super::*;

#[derive(Copy, Clone, Debug, PartialEq, ValueEnum)]
pub(crate) enum UseColor {
  Auto,
  Always,
  Never,
}
