#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub(crate) enum Delimiter {
  Brace,
  Bracket,
  Paren,
}

impl Delimiter {
  pub(crate) fn open(self) -> char {
    match self {
      Self::Brace => '{',
      Self::Bracket => '[',
      Self::Paren => '(',
    }
  }

  pub(crate) fn close(self) -> char {
    match self {
      Self::Brace => '}',
      Self::Bracket => ']',
      Self::Paren => ')',
    }
  }
}
