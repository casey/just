use super::*;

pub(crate) struct Numerator {
  bindings: u32,
  recipes: u32,
}

impl Numerator {
  pub(crate) fn new() -> Self {
    Self {
      bindings: constants().len().try_into().unwrap(),
      recipes: 0,
    }
  }

  pub(crate) fn next_binding(&mut self) -> Number {
    let id = self.bindings;
    self.bindings += 1;
    Number(id)
  }

  pub(crate) fn next_recipe(&mut self) -> Number {
    let id = self.recipes;
    self.recipes += 1;
    Number(id)
  }

  pub(crate) fn constant(i: usize) -> Number {
    assert!(i < constants().len());
    Number(i.try_into().unwrap())
  }
}
