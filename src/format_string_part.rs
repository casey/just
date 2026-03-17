#[derive(PartialEq, Debug, Clone, Ord, Eq, PartialOrd)]
pub(crate) enum FormatStringPart {
  Continue,
  End,
  Single,
  Start,
}
