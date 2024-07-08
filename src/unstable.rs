#[derive(Copy, Clone, Debug, PartialEq, Ord, Eq, PartialOrd)]
pub(crate) enum Unstable {
  Modules,
}

impl Unstable {
  pub(crate) fn message(self) -> String {
    match self {
      Self::Modules => "Modules are currently unstable.".into(),
    }
  }
}
