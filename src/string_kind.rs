#[derive(Clone, Copy)]
pub(crate) enum StringKind {
  Backtick,
  Cooked,
  Raw,
}

impl StringKind {
  pub(crate) fn delimiter(self) -> char {
    match self {
      Self::Backtick => '`',
      Self::Cooked => '"',
      Self::Raw => '\'',
    }
  }
}
