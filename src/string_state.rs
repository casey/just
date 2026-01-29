use super::*;

pub(crate) enum StringState {
  FormatContinue(StringKind),
  FormatStart,
  Normal,
}
