pub trait Ordinal {
  /// Convert an index starting at 0 to an ordinal starting at 1
  fn ordinal(self) -> Self;
}

impl Ordinal for usize {
  fn ordinal(self) -> Self {
    self + 1
  }
}
