use super::*;

pub(crate) struct Numerator(u32);

impl Numerator {
  pub(crate) fn new() -> Self {
    Self(constants().len().try_into().unwrap())
  }

  pub(crate) fn next(&mut self) -> Id {
    let id = self.0;
    self.0 += 1;
    Id(id)
  }

  pub(crate) fn constant(i: usize) -> Id {
    assert!(i < constants().len());
    Id(i.try_into().unwrap())
  }
}
