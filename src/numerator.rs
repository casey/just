use super::*;

pub(crate) struct Numerator(u32);

impl Numerator {
  pub(crate) fn new() -> Self {
    Self(constants().len().try_into().unwrap())
  }

  pub(crate) fn next(&mut self) -> Number {
    let id = self.0;
    self.0 += 1;
    Number(id)
  }

  pub(crate) fn constant(i: usize) -> Number {
    assert!(i < constants().len());
    Number(i.try_into().unwrap())
  }
}
