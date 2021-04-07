#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub(crate) enum Delimiter {
  Brace,
  Bracket,
  Interpolation,
  Paren,
}

impl Delimiter {
  pub(crate) fn open(self) -> &'static str {
    match self {
      Self::Brace => "{",
      Self::Bracket => "[",
      Self::Interpolation => "{{",
      Self::Paren => "(",
    }
  }

  pub(crate) fn close(self) -> &'static str {
    match self {
      Self::Brace => "}",
      Self::Bracket => "]",
      Self::Interpolation => "}}",
      Self::Paren => ")",
    }
  }
}
