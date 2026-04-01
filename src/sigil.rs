#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Sigil {
  Guard,
  Infallible,
  Quiet,
}
